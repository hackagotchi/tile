use hexa::{Renderer, Scene};

pub struct DynamicScene {
    scene: Box<dyn Scene>,
    #[allow(dead_code)]
    /// It's important to keep this library alive for as long as the Scene is,
    /// the vtable there points to functions inside of this library.
    lib: libloading::Library,
}
impl std::ops::Deref for DynamicScene {
    type Target = Box<dyn Scene>;

    fn deref(&self) -> &Self::Target {
        &self.scene
    }
}
impl std::ops::DerefMut for DynamicScene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene
    }
}
impl DynamicScene {
    pub fn new(r: &mut dyn Renderer) -> Self {
        type AddFunc = unsafe fn(&mut dyn hexa::Renderer) -> *mut dyn Scene;

        let lib = libloading::Library::new("../hackstead_scene/target/debug/libhackstead_scene.so")
            .unwrap();
        Self {
            scene: unsafe {
                let init: libloading::Symbol<AddFunc> = lib.get(b"_scene_init").unwrap();
                Box::from_raw(init(r))
            },
            lib,
        }
    }
}
