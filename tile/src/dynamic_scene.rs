use libloading::{Library, Symbol};
use hexa::{Renderer, Scene};

pub struct LoadedSceneLib {
    scene: Box<dyn Scene>,
    #[allow(dead_code)]
    /// It's important to keep this library alive for as long as the Scene is,
    /// the vtable there points to functions inside of this library.
    lib: Option<Library>,
}
impl LoadedSceneLib {
    fn load(r: &mut dyn Renderer) -> Self {
        type InitFunc = unsafe fn(&mut dyn Renderer) -> *mut dyn Scene;

        const PATH: &'static str = "../target/debug/deps/libhackstead_scene.so";
        let lib = Library::new(PATH).unwrap();
        Self {
            scene: unsafe {
                let init: Symbol<InitFunc> = lib.get(b"_scene_init").unwrap();
                Box::from_raw(init(r))
            },
            lib: Some(lib),
        }
    }
}

pub struct DynamicScene {
    loaded_lib: Option<LoadedSceneLib>
}
impl std::ops::Deref for DynamicScene {
    type Target = Box<dyn Scene>;

    fn deref(&self) -> &Self::Target {
        &self.loaded_lib.as_ref().unwrap().scene
    }
}
impl std::ops::DerefMut for DynamicScene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.loaded_lib.as_mut().unwrap().scene
    }
}
impl DynamicScene {
    pub fn new(r: &mut dyn Renderer) -> Self {
        Self {
            loaded_lib: Some(LoadedSceneLib::load(r)),
        }
    }
    pub fn reloaded(&mut self, r: &mut dyn Renderer) -> Self {
        drop(self.loaded_lib.take());
        Self::new(r)
    }
}
