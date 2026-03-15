use crate::state::config::AppConfig;
use crate::libs::device_manager::DeviceManager;
use rodio::{ OutputStream, OutputStreamHandle, Sink };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::time::Instant;

static AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();
static MOUSE_AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();

#[derive(Clone)]
pub struct AudioContext {
    // Wrapped in Arc<Mutex<...>> so the stream can be hot-swapped when the OS
    // default audio output device changes without recreating the whole context.
    _stream: Arc<Mutex<OutputStream>>,
    pub(crate) stream_handle: Arc<Mutex<OutputStreamHandle>>,
    pub(crate) keyboard_samples: Arc<Mutex<Option<(Vec<f32>, u16, u32)>>>,
    pub(crate) mouse_samples: Arc<Mutex<Option<(Vec<f32>, u16, u32)>>>,
    pub(crate) key_map: Arc<Mutex<HashMap<String, Vec<[f32; 2]>>>>,
    pub(crate) mouse_map: Arc<Mutex<HashMap<String, Vec<[f32; 2]>>>>,
    pub(crate) max_voices: usize,
    pub(crate) key_pressed: Arc<Mutex<HashMap<String, bool>>>,
    pub(crate) mouse_pressed: Arc<Mutex<HashMap<String, bool>>>,
    pub(crate) key_sinks: Arc<Mutex<HashMap<String, Sink>>>,
    pub(crate) mouse_sinks: Arc<Mutex<HashMap<String, Sink>>>,
    pub(crate) device_manager: DeviceManager,
    // Timing tracking for rapid event detection
    pub(crate) last_keyboard_sound_time: Arc<Mutex<Option<Instant>>>,
    pub(crate) last_mouse_sound_time: Arc<Mutex<Option<Instant>>>,
    // Tracks the OS default device name so we can detect when it changes
    pub(crate) last_default_device: Arc<Mutex<Option<String>>>,
}

// Manual PartialEq implementation for component compatibility
impl PartialEq for AudioContext {
    fn eq(&self, other: &Self) -> bool {
        // For component props, we consider AudioContext instances equal if they're the same Arc
        Arc::ptr_eq(&self._stream, &other._stream)
    }
}

impl AudioContext {
    pub fn new() -> Self {
        // Initialize device manager
        let device_manager = DeviceManager::new();
        let config = AppConfig::load();

        // Try to use selected device or fall back to default
        let (stream, stream_handle) = match &config.selected_audio_device {
            Some(device_id) => {
                match device_manager.get_output_device_by_id(device_id) {
                    Ok(Some(device)) => {
                        match rodio::OutputStream::try_from_device(&device) {
                            Ok((stream, handle)) => (stream, handle),
                            Err(e) => {
                                eprintln!(
                                    "❌ Failed to create stream from selected device {}: {}",
                                    device_id,
                                    e
                                );
                                eprintln!("🔄 Falling back to default device...");
                                rodio::OutputStream
                                    ::try_default()
                                    .expect("Failed to create default audio output stream")
                            }
                        }
                    }
                    Ok(None) => {
                        eprintln!("❌ Selected audio device {} not found, using default", device_id);
                        rodio::OutputStream
                            ::try_default()
                            .expect("Failed to create default audio output stream")
                    }
                    Err(e) => {
                        eprintln!("❌ Error accessing selected device {}: {}", device_id, e);
                        rodio::OutputStream
                            ::try_default()
                            .expect("Failed to create default audio output stream")
                    }
                }
            }
            None => {
                rodio::OutputStream
                    ::try_default()
                    .expect("Failed to create default audio output stream")
            }
        };

        // Remember which device we actually opened so we can detect OS changes later
        let initial_default_name = match &config.selected_audio_device {
            None => device_manager.get_default_output_device_name(),
            Some(_) => None, // Tracking not needed when a specific device is pinned
        };

        let context = Self {
            _stream: Arc::new(Mutex::new(stream)),
            stream_handle: Arc::new(Mutex::new(stream_handle)),
            keyboard_samples: Arc::new(Mutex::new(None)),
            mouse_samples: Arc::new(Mutex::new(None)),
            key_map: Arc::new(Mutex::new(HashMap::new())),
            mouse_map: Arc::new(Mutex::new(HashMap::new())),
            max_voices: 20, // Increased max voices to reduce audio interruptions
            key_pressed: Arc::new(Mutex::new(HashMap::new())),
            mouse_pressed: Arc::new(Mutex::new(HashMap::new())),
            key_sinks: Arc::new(Mutex::new(HashMap::new())),
            mouse_sinks: Arc::new(Mutex::new(HashMap::new())),
            device_manager,
            last_keyboard_sound_time: Arc::new(Mutex::new(None)),
            last_mouse_sound_time: Arc::new(Mutex::new(None)),
            last_default_device: Arc::new(Mutex::new(initial_default_name)),
        };
        // Initialize volume from config
        let config = AppConfig::load();
        AUDIO_VOLUME.get_or_init(|| Mutex::new(config.volume));
        MOUSE_AUDIO_VOLUME.get_or_init(|| Mutex::new(config.mouse_volume));

        // Load soundpack from config
        match super::soundpack_loader::load_soundpack(&context) {
            Ok(_) => {}
            Err(e) => eprintln!("❌ Failed to load initial soundpack: {}", e),
        }

        context
    }
    pub fn set_volume(&self, volume: f32) {
        // Update volume for current keys only
        let key_sinks = self.key_sinks.lock().unwrap();
        for sink in key_sinks.values() {
            sink.set_volume(volume);
        }

        // Update global variable
        if let Some(global) = AUDIO_VOLUME.get() {
            let mut g = global.lock().unwrap();
            *g = volume;
        }

        // Save to config file
        let mut config = AppConfig::load();
        config.volume = volume;
        let _ = config.save();
    }

    pub fn get_volume(&self) -> f32 {
        AUDIO_VOLUME.get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(1.0)
    }

    pub fn set_mouse_volume(&self, volume: f32) {
        // Update volume for current mouse events only
        let mouse_sinks = self.mouse_sinks.lock().unwrap();
        for sink in mouse_sinks.values() {
            sink.set_volume(volume);
        }

        // Update global variable
        if let Some(global) = MOUSE_AUDIO_VOLUME.get() {
            let mut g = global.lock().unwrap();
            *g = volume;
        }

        // Save to config file
        let mut config = AppConfig::load();
        config.mouse_volume = volume;
        let _ = config.save();
    }

    pub fn get_mouse_volume(&self) -> f32 {
        MOUSE_AUDIO_VOLUME.get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(1.0)
    }
    pub fn create_with_device(device_id: Option<String>) -> Result<Self, String> {
        // Initialize device manager
        let device_manager = DeviceManager::new();

        // Create stream with selected device
        let (stream, stream_handle) = match &device_id {
            Some(id) => {
                match device_manager.get_output_device_by_id(id) {
                    Ok(Some(device)) => {
                        match rodio::OutputStream::try_from_device(&device) {
                            Ok((stream, handle)) => (stream, handle),
                            Err(e) => {
                                eprintln!("❌ Failed to create stream from device {}: {}", id, e);
                                return Err(format!("Failed to use device: {}", e));
                            }
                        }
                    }
                    Ok(None) => {
                        eprintln!("❌ Device {} not found", id);
                        return Err(format!("Device not found: {}", id));
                    }
                    Err(e) => {
                        eprintln!("❌ Error accessing device {}: {}", id, e);
                        return Err(format!("Error accessing device: {}", e));
                    }
                }
            }
            None => {
                rodio::OutputStream
                    ::try_default()
                    .map_err(|e| format!("Failed to create default stream: {}", e))?
            }
        };

        let context = Self {
            _stream: Arc::new(Mutex::new(stream)),
            stream_handle: Arc::new(Mutex::new(stream_handle)),
            keyboard_samples: Arc::new(Mutex::new(None)),
            mouse_samples: Arc::new(Mutex::new(None)),
            key_map: Arc::new(Mutex::new(HashMap::new())),
            mouse_map: Arc::new(Mutex::new(HashMap::new())),
            max_voices: 20, // Increased max voices to reduce audio interruptions
            key_pressed: Arc::new(Mutex::new(HashMap::new())),
            mouse_pressed: Arc::new(Mutex::new(HashMap::new())),
            key_sinks: Arc::new(Mutex::new(HashMap::new())),
            mouse_sinks: Arc::new(Mutex::new(HashMap::new())),
            device_manager,
            last_keyboard_sound_time: Arc::new(Mutex::new(None)),
            last_mouse_sound_time: Arc::new(Mutex::new(None)),
            last_default_device: Arc::new(Mutex::new(None)),
        };

        // Initialize volume from config
        let config = AppConfig::load();
        AUDIO_VOLUME.get_or_init(|| Mutex::new(config.volume));
        MOUSE_AUDIO_VOLUME.get_or_init(|| Mutex::new(config.mouse_volume)); // Load soundpack from config
        match super::soundpack_loader::load_soundpack(&context) {
            Ok(_) => {}
            Err(e) => eprintln!("❌ Failed to load initial soundpack: {}", e),
        }

        Ok(context)
    }

    pub fn get_current_device_info(&self) -> Option<String> {
        let config = AppConfig::load();
        config.selected_audio_device
    }

    pub fn test_current_device(&self) -> bool {
        let config = AppConfig::load();
        match &config.selected_audio_device {
            Some(device_id) => {
                self.device_manager.test_output_device(device_id).unwrap_or(false)
            }
            None => true, // Default device is always considered available
        }
    }

    /// Recreate the OutputStream / OutputStreamHandle in-place, pointing at
    /// whichever device the current config selects (or the OS default when none
    /// is configured).  Stale sinks and pressed-key state are cleared so the
    /// next key/mouse event will open fresh sinks on the new stream.
    pub fn reinitialize_stream(&self) {
        let config = AppConfig::load();
        let device_manager = DeviceManager::new();

        let result: Option<(OutputStream, OutputStreamHandle)> = match &config.selected_audio_device {
            Some(device_id) => {
                match device_manager.get_output_device_by_id(device_id) {
                    Ok(Some(device)) => rodio::OutputStream::try_from_device(&device).ok(),
                    _ => rodio::OutputStream::try_default().ok(),
                }
            }
            None => rodio::OutputStream::try_default().ok(),
        };

        if let Some((new_stream, new_handle)) = result {
            // Update tracked default device name (only meaningful when using OS default)
            let new_default_name = if config.selected_audio_device.is_none() {
                device_manager.get_default_output_device_name()
            } else {
                None
            };

            // Swap stream + handle atomically
            *self._stream.lock().unwrap() = new_stream;
            *self.stream_handle.lock().unwrap() = new_handle;

            // Discard stale sinks — they are tied to the old stream
            self.key_sinks.lock().unwrap().clear();
            self.mouse_sinks.lock().unwrap().clear();
            self.key_pressed.lock().unwrap().clear();
            self.mouse_pressed.lock().unwrap().clear();

            *self.last_default_device.lock().unwrap() = new_default_name;
            println!("🔄 Audio stream reinitialized for new default output device");
        } else {
            eprintln!("❌ Failed to reinitialize audio stream");
        }
    }

    /// Poll whether the OS default output device has changed since the stream
    /// was last (re)initialized and, if so, reinitialize automatically.
    /// This is a no-op when the user has explicitly pinned a specific device.
    pub fn check_and_reinitialize_if_default_changed(&self) {
        let config = AppConfig::load();
        if config.selected_audio_device.is_some() {
            // User explicitly selected a device — don't override their choice.
            return;
        }

        let device_manager = DeviceManager::new();
        let current_default = device_manager.get_default_output_device_name();
        let last_known = self.last_default_device.lock().unwrap().clone();

        if current_default != last_known {
            println!(
                "🔄 Default audio device changed ({:?} → {:?}), reinitializing stream…",
                last_known,
                current_default
            );
            self.reinitialize_stream();
        }
    }
}
