use std::default;

use crate::core::libretro;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[repr(u32)]
#[derive(Debug, Clone, Default, FromPrimitive)]
enum Rotation { #[default]Upright = 0, RotateLeft = 1, UpsideDown = 2, RotateRight = 3 }

fn safe_read<T>(ptr: *const std::ffi::c_void) -> Option<T> {
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { std::ptr::read(ptr as *const T) })
    }
}

#[derive(Debug, Clone, Default)]
struct RetroMessageExt {
    msg: String,
    duration: u32,
    priority: u32,
    level: libretro::retro_log_level,
    type_: libretro::retro_message_type,
    progress: i8,
}
impl Default for libretro::retro_log_level {
    fn default() -> Self {
        libretro::retro_log_level::RETRO_LOG_INFO
    }
}
impl Default for libretro::retro_message_type {
    fn default() -> Self {
        libretro::retro_message_type::RETRO_MESSAGE_TYPE_NOTIFICATION
    }
}

#[derive(Debug, Clone, Default)]
pub struct RetroEnvironment {
    rotation: Option<Rotation>,
    pixel_format: Option<libretro::retro_pixel_format>,
    toast_messages: Vec<RetroMessageExt>,
    keyboard_callback: libretro::retro_keyboard_event_t,
    disk_control_callback: Option<libretro::retro_disk_control_callback>,
    disk_control_ext_callback: Option<libretro::retro_disk_control_ext_callback>,
    async_audio_callback: Option<libretro::retro_audio_callback>,
    support_no_game: bool,
}

impl RetroEnvironment {
    pub fn handle_environment_call(&mut self, command: u32, data: *mut std::ffi::c_void) -> bool {
        #[allow(non_snake_case)]
        match command {
            // common GET commands

            // media related calls
            libretro::RETRO_ENVIRONMENT_SET_ROTATION => {
                if let Some(rot) = safe_read::<u32>(data) {
                    if let Some(rot) = Rotation::from_u32(rot) {
                        self.set_rotation(rot)
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            libretro::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
                self.pixel_format = safe_read::<libretro::retro_pixel_format>(data);
                true
            },
            libretro::RETRO_ENVIRONMENT_SET_FRAME_TIME_CALLBACK => {
                false
            },
            libretro::RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK => {
                let callback = unsafe { std::ptr::read(data as *const libretro::retro_audio_callback) };
                self.async_audio_callback = Some(callback);
                true
            }

            // toast message calls
            libretro::RETRO_ENVIRONMENT_SET_MESSAGE => {
                let msg: libretro::retro_message = unsafe { std::ptr::read(data as *const libretro::retro_message) };
                let msg_ext = RetroMessageExt {
                    msg: unsafe { std::ffi::CStr::from_ptr(msg.msg).to_string_lossy().into_owned() },
                    duration: msg.frames,
                    ..Default::default()
                };
                self.push_toasts(msg_ext)
            },
            libretro::RETRO_ENVIRONMENT_SET_MESSAGE_EXT => {
                let msg: libretro::retro_message_ext = unsafe { std::ptr::read(data as *const libretro::retro_message_ext) };
                let msg_ext = RetroMessageExt {
                    msg: unsafe { std::ffi::CStr::from_ptr(msg.msg).to_string_lossy().into_owned() },
                    duration: msg.duration,
                    priority: msg.priority,
                    level: msg.level,
                    type_: msg.type_,
                    progress: msg.progress,
                };
                if msg.target==libretro::retro_message_target::RETRO_MESSAGE_TARGET_OSD {
                    self.push_toasts(msg_ext)
                } else {
                    println!("{:?}", msg_ext);
                    true
                }
            },

            // system-related calls
            libretro::RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL => {
                let level = unsafe { std::ptr::read(data as *const u32) };
                println!("Performance level: {}", level);
                true
            }, //NOTE: call at game-level
            libretro::RETRO_ENVIRONMENT_SET_KEYBOARD_CALLBACK => {
                let callback = unsafe { std::ptr::read(data as *const libretro::retro_keyboard_callback) };
                self.keyboard_callback = callback.callback;
                true
            },
            libretro::RETRO_ENVIRONMENT_SET_DISK_CONTROL_INTERFACE => {
                let callback = unsafe { std::ptr::read(data as *const libretro::retro_disk_control_callback) };
                self.disk_control_callback = Some(callback);
                true
            },
            libretro::RETRO_ENVIRONMENT_SET_DISK_CONTROL_EXT_INTERFACE => {
                let callback = unsafe { std::ptr::read(data as *const libretro::retro_disk_control_ext_callback) };
                self.disk_control_ext_callback = Some(callback);
                true
            },
            libretro::RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME => {
                let flag = unsafe{ std::ptr::read(data as *const bool) };
                self.support_no_game = flag;
                true
            }

            // core options register
            libretro::RETRO_ENVIRONMENT_SET_VARIABLES => {
                //struct retro_variable *
                true
            },
            libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS => {
                //struct retro_core_option_definition *
                true
            },
            libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL => {
                //struct retro_core_options_intl *
                true
            },
            libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2 => {
                //struct retro_core_options_v2 *
                true
            },
            libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL => {
                //struct retro_core_options_v2_intl *
                true
            },
            
            // HW render calls
            libretro::RETRO_ENVIRONMENT_SET_HW_RENDER => false,
            libretro::RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE => false,
            libretro::RETRO_ENVIRONMENT_SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE => false,
            libretro::RETRO_ENVIRONMENT_SET_HW_SHARED_CONTEXT => false,
            libretro::RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER => false,
            libretro::RETRO_ENVIRONMENT_GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT => false,

            // unsupported or unknown commands
            libretro::RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS => false,
            libretro::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY => false,
            libretro::RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS => false,
            _ => false,
        }
    }

    pub fn set_rotation(&mut self, rot: Rotation) -> bool {
        self.rotation = Some(rot);
        true
    }

    pub fn push_toasts(&mut self, msg_ext: RetroMessageExt) -> bool {
        self.toast_messages.push(msg_ext);
        true
    }

}
