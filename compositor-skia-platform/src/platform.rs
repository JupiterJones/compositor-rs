#[derive(Debug, Clone)]
pub enum Platform {
    #[cfg(target_os = "macos")]
    Metal(crate::MetalPlatform),
    #[cfg(target_os = "windows")]
    Angle(crate::OpenGLPlatform),
    #[cfg(all(target_os = "linux", any(feature = "x11", feature = "wayland")))]
    OpenGL(crate::OpenGLPlatform),
    Unsupported,
}

impl Platform {
    #[cfg(target_os = "macos")]
    pub fn try_as_metal_platform(&self) -> Option<&crate::MetalPlatform> {
        match self {
            Platform::Metal(platform) => Some(platform),
            _ => None,
        }
    }

    #[cfg(any(
        target_os = "windows",
        all(target_os = "linux", any(feature = "x11", feature = "wayland"))
    ))]
    pub fn try_as_opengl_platform(&self) -> Option<&crate::OpenGLPlatform> {
        match self {
            #[cfg(target_os = "windows")]
            Platform::Angle(platform) => Some(platform),
            #[cfg(all(target_os = "linux", any(feature = "x11", feature = "wayland")))]
            Platform::OpenGL(platform) => Some(platform),
            _ => None,
        }
    }
}
