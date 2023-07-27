use std::ffi;

use ash::vk;

#[macro_export]
macro_rules! VK_CHECK {
    ($x:expr) => {
        match $x {
            result => {
                if result != vk::Result::SUCCESS {
                    println!("Detected Vulkan error: {:?}", result);
                    unsafe {
                        std::ptr::nul::<i32>() as *const i32;
                    }
                }
            }
        }
    };
}
