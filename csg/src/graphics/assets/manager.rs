use std::{any::{Any, TypeId}, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData, sync::Arc};
use crate::{graphics::assets::{AssetDefault, JsonLoadable, ReloadableAsset}, prelude::*};
use ahash::AHashMap;
use slotmap::{Key, KeyData, SlotMap};
use unsafe_any::UnsafeAnyExt;

/// Asset Key for asset of type T
pub struct AssetKey<T>{
    key: KeyData,
    _phantom: PhantomData<T>
} impl<T> AssetKey<T>{
    /// Cast one key type into another
    /// Marked unsafe as blindly casting keys can lead to crashes or UB
    unsafe fn type_crunch<O>(self) -> AssetKey<O>{
        AssetKey {key: self.key, _phantom: PhantomData}
    }
} impl<T> Copy for AssetKey<T>{
} impl<T> Clone for AssetKey<T>{
    /// Clone a key (See [`core::clone::Clone`])
    /// Manually implemented to prevent unnececary requirements on `T`
    fn clone(&self) -> Self {
        AssetKey {
            key: self.key.clone(),
            _phantom: PhantomData
        }
    }
} impl<T> Default for AssetKey<T>{
    /// Default value of a key (See [`core::default::Default`])
    /// Manually implemented to prevent unnececary requirements on `T`
    fn default() -> Self {
        AssetKey {
            key: KeyData::default(),
            _phantom: PhantomData
        }
    }
} impl<T> Eq for AssetKey<T> {
} impl<T> PartialEq for AssetKey<T>{
    /// Check if two keys are equal (See [`core::cmp::PartialEq`])
    /// Manually implemented to prevent unnececary requirements on `T`
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
} impl<T> Ord for AssetKey<T>{
    /// Compare two keys (See [`core::cmp::Ord`])
    /// Manually implemented to prevent unnececary requirements on `T`
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
} impl<T> PartialOrd for AssetKey<T>{
    /// Compare two keys (See [`core::cmp::PartialOrd`])
    /// Manually implemented to prevent unnececary requirements on `T`
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
} impl<T> Hash for AssetKey<T>{
    /// Hash a key (See [`core::hash::Hash`])
    /// Manually implemented to prevent unnececary requirements on `T`
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
} impl<T> Debug for AssetKey<T>{
    /// Print Debug information for a key (See [`core::fmt::Debug`])
    /// Manually implemented to prevent unnececary requirements on `T`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.key.fmt(f)
    }
} impl<T> From<KeyData> for AssetKey<T>{
    /// Create an AssetKey from raw Keydata
    /// Manually implemented to add the `T`
    fn from(value: KeyData) -> Self {
        AssetKey { key: value, _phantom: PhantomData }
    }
} unsafe impl<T> slotmap::Key for AssetKey<T>{
    /// Get the Keydata of an AssetKey
    /// Manually implemented to add the `T`
    fn data(&self) -> KeyData {
        self.key
    }
}


/// CSG Asset Manager,
/// Assets are stored and indexed by thier type and AssetKey<T>
/// Allowing consumers to only store a copyable AssetKey and only fetch when needed to
pub struct AssetManager{
    assets: AHashMap<TypeId, (HashMap<String, AssetKey<()>>, SlotMap<AssetKey<()>, Box<dyn ReloadableAsset>>)>,
    defaults: AHashMap<TypeId, Box<dyn Any>>,
    vfs: Arc<dyn CasinoFS>
} impl AssetManager{
    /// Create the asset manager, should only be done once!
    pub fn create(vfs: Arc<dyn CasinoFS>) -> AssetManager{
        let mut defaults = AHashMap::new();
        for def in inventory::iter::<AssetDefault>{
            defaults.insert(def.key, (def.value)());
        }

        AssetManager {
            assets: AHashMap::new(),
            defaults,
            vfs
        }
    }

    /// Set the default value for assets of type T
    /// # Returns
    /// Some(T) if there was a previous default
    /// None if there was no previous default
    pub fn set_default<T: 'static>(&mut self, value: T) -> Option<T>{
        match self.defaults.insert(TypeId::of::<T>(), Box::new(value)){
            #[allow(unstable_name_collisions)]
            Some(any) => Some(unsafe {*any.downcast_unchecked()}),
            None => None
        }
    }

    /// Set the default value for assets of type T
    /// # Returns
    /// Some(T) if there is a default
    /// None if there is no default
    pub fn get_default<T: 'static>(&self) -> Option<&T>{
        match self.defaults.get(&TypeId::of::<T>()){
            Some(any) => Some(unsafe {*any.downcast_ref_unchecked()}),
            None => None
        }
    }

    /// Load an asset from a path on the vfs
    /// Returns Err if it cant read the path or F fails
    /// Returns Ok if it works
    fn load_from_vfs<T, F: FnOnce(&[u8]) -> GResult<T>>(vfs: &Arc<dyn CasinoFS>, path: &str, f: F) -> GResult<T>{
        let data = vfs.read_all(path).g_err()?;
        f(data)
    }

    /// Load an asset from a path on the vfs
    /// Returns Err if it cant read the path or F fails
    /// Returns Ok if it works
    fn load_json_from_vfs<'a, T: JsonLoadable<'a>>(vfs: &Arc<dyn CasinoFS>, path: &str) -> GResult<T>{
        let data = vfs.read_all(path).g_err()?;
        T::load(data)
    }

    /// Reload an asset from a path on the vfs
    /// Returns Err if it cant read the path or instance.reload fails
    /// Returns Ok if it works
    fn reload_from_vfs<T: ReloadableAsset>(vfs: &Arc<dyn CasinoFS>, path: &str, instance: &mut T) -> GResult<()>{
        let data = vfs.read_all(path).g_err()?;
        instance.reload(data)
    }


    /// Creates an asset of type T by loading it from `path` in the VFS
    /// If not loaded, atempt to load the asset returning the asset key
    /// If already loaded, simply return the first key
    /// Reurns GResult::Err if the asset cannot be (re)loaded
    /// Returns GResult::Ok otherwise
    pub fn create_asset<T: ReloadableAsset + 'static, F: FnOnce(&[u8]) -> GResult<T>>(&mut self, path: &str, f: F) -> GResult<AssetKey<T>>{
        // Pull the entry in the table (create it if it doesnt exist)
        let entry = self.assets.entry(TypeId::of::<T>());
        let (by_name, entries) = entry.or_insert_with(|| (HashMap::new(), SlotMap::with_key()));
        
        // If we already created the asset
        if let Some(key) = by_name.get(path){
            Ok(unsafe{key.type_crunch()})
        } else {
            // Create it
            let obj = Self::load_from_vfs::<T, F>(&self.vfs, path, f)?;
            let gkey = entries.insert(Box::new(obj));
            by_name.insert(path.to_string(), gkey.clone());

            Ok(unsafe{gkey.type_crunch()})
        }
    }

    pub fn create_json_asset<'a, T: ReloadableAsset + JsonLoadable<'a> + 'static>(&mut self, path: &str) -> GResult<AssetKey<T>>{
        // Pull the entry in the table (create it if it doesnt exist)
        let entry = self.assets.entry(TypeId::of::<T>());
        let (by_name, entries) = entry.or_insert_with(|| (HashMap::new(), SlotMap::with_key()));
        
        // If we already created the asset
        if let Some(key) = by_name.get(path){
            Ok(unsafe{key.type_crunch()})
        } else {
            // Create it
            let obj = Self::load_json_from_vfs::<T>(&self.vfs, path)?;
            let gkey = entries.insert(Box::new(obj));
            by_name.insert(path.to_string(), gkey.clone());

            Ok(unsafe{gkey.type_crunch()})
        }
    }


    /// Fetch an asset of type T by its AssetKey<T>
    /// if Key is null and a default exists it returns it
    /// Returns Some if the asset exists
    /// Returns None if it cannot be found
    pub fn get_asset<T: 'static>(&self, key: AssetKey<T>) -> Option<&T>{
        if key.is_null(){
            return self.get_default()
        }

        // Pull the entries slotmap from the table returning early if it doesnt exist
        let entries = &self.assets.get(&TypeId::of::<T>())?.1;

        // Pull the asset entry from the slotmap of entries returning early if it doesnt exist
        // SAFTEY: Slotkey type must be T for Hashmap[t.typid]
        let entry = entries.get(unsafe{key.type_crunch()})?;

        // Cast it to the right type
        // SAFTEY: Slotmap conents must by of the type of the Typeid
        Some(unsafe{entry.downcast_ref_unchecked()})
    }

    /// Fetch an asset of type T by its AssetKey<T>
    /// if Key is null and a default exists it returns it
    /// Returns Some if the asset exists or a default value is defined
    /// Returns None if it cannot be found and there is no default
    pub fn get_asset_or_default<T: 'static>(&self, key: AssetKey<T>) -> Option<&T>{
        self.get_asset(key).or_else(|| self.get_default())
    }

    /// Fetch a mutable reference of an asset of type T by its AssetKey<T>
    /// Returns Some if the asset exists
    /// Returns None if it cannot be found
    pub fn get_asset_mut<T: 'static>(&mut self, key: AssetKey<T>) -> Option<&mut T>{
        // Pull the entries slotmap from the table returning early if it doesnt exist
        let entries = &mut self.assets.get_mut(&TypeId::of::<T>())?.1;

        // Pull the asset entry from the slotmap of entries returning early if it doesnt exist
        // SAFTEY: Slotkey type must be T for Hashmap[t.typid]
        let entry = entries.get_mut(unsafe{key.type_crunch()})?;

        // Cast it to the right type
        // SAFTEY: Slotmap conents must by of the type of the Typeid
        Some(unsafe{entry.downcast_mut_unchecked()})
    }

    /// Fetch an asset and its coresponding key of type T by its path
    /// Returns Some if the asset exists
    /// Returns None if it cannot be found
    pub fn get_asset_by_path<T: 'static>(&self, path: &str) -> Option<(AssetKey<T>, &T)>{
        // Pull the entries hashmap and slotmap from the table returning early if it doesnt exist
        let (by_name,entries) = self.assets.get(&TypeId::of::<T>())?;

        let gkey = *by_name.get(path)?;

        // Pull the asset entry from the slotmap of entries
        // SAFTEY: Hashmap must match slotmaps contents
        let entry = unsafe{entries.get_unchecked(gkey)};

        // Cast it to the right type
        // SAFTEY: Slotmap conents must by of the type of the Typeid
        Some(unsafe{(gkey.type_crunch(), entry.downcast_ref_unchecked())})
    }
    
    /// Fetch mutable reference of an asset and its coresponding key of type T by its path
    /// Returns Some if the asset exists
    /// Returns None if it cannot be found
    pub fn get_asset_mut_by_path<T: 'static>(&mut self, path: &str) -> Option<(AssetKey<T>, &mut T)>{
        // Pull the entries hashmap and slotmap from the table returning early if it doesnt exist
        let (by_name,entries) = self.assets.get_mut(&TypeId::of::<T>())?;

        let gkey = *by_name.get(path)?;

        // Pull the asset entry from the slotmap of entries
        // SAFTEY: Hashmap must match slotmaps contents
        let entry = unsafe{entries.get_unchecked_mut(gkey)};

        // Cast it to the right type
        // SAFTEY: Slotmap conents must by of the type of the Typeid
        Some(unsafe{(gkey.type_crunch(), entry.downcast_mut_unchecked())})
    }

    /// Fetch an asset of type T by its path, if it cannot be found, try to create it
    /// If not loaded, atempt to load the asset returning a reference and its key
    /// If already loaded, simply return a reference and the first key
    /// Reurns Err if the asset cannot be loaded
    /// Returns Ok otherwise
    pub fn get_or_create_asset<T: ReloadableAsset + 'static, F: FnOnce(&[u8]) -> GResult<T>>(&mut self, path: &str, f: F) -> GResult<(AssetKey<T>, &T)>{
        // Pull the entry in the table (create it if it doesnt exist)
        let t_entry = self.assets.entry(TypeId::of::<T>());
        let (by_name, entries) = t_entry.or_insert_with(|| (HashMap::new(), SlotMap::with_key()));

        // If we already created the asset
        if let Some(key) = by_name.get(path){
            // Pull the asset entry from the slotmap of entries
            // SAFTEY: Hashmap must match slotmaps contents
            let entry = unsafe{entries.get_unchecked_mut(*key)};

            // Cast it to the right type
            // SAFTEY: Slotmap conents must by of the type of the Typeid
            GResult::Ok(unsafe{(key.type_crunch(), entry.downcast_mut_unchecked())})
        } else {
            let obj = Self::load_from_vfs::<T, F>(&self.vfs, path, f)?;
            let gkey = entries.insert(Box::new(obj));
            by_name.insert(path.to_string(), gkey.clone());

            let entry = unsafe{entries.get_unchecked(gkey)};

            GResult::Ok(unsafe{(gkey.type_crunch(), entry.downcast_ref_unchecked())})
        }
    }

    /// Fetch a mutable reference of an asset of type T by its path, if it cannot be found, try to create it
    /// If not loaded, atempt to load the asset returning a mutable reference and its key
    /// If already loaded, simply return a mutable reference and the first key
    /// Reurns GResult::Err if the asset cannot be (re)loaded
    /// Returns GResult::Ok otherwise
    pub fn get_or_create_asset_mut<T: ReloadableAsset + 'static, F: FnOnce(&[u8]) -> GResult<T>>(&mut self, path: &str, f: F) -> GResult<(AssetKey<T>, &mut T)>{
        // Pull the entry in the table (create it if it doesnt exist)
        let t_entry = self.assets.entry(TypeId::of::<T>());
        let (by_name, entries) = t_entry.or_insert_with(|| (HashMap::new(), SlotMap::with_key()));

        // If we already created the asset
        if let Some(key) = by_name.get(path){
            // Pull the asset entry from the slotmap of entries
            // SAFTEY: Hashmap must match slotmaps contents
            let entry = unsafe{entries.get_unchecked_mut(*key)};

            // Cast it to the right type
            // SAFTEY: Slotmap conents must by of the type of the Typeid
            Ok(unsafe{(key.type_crunch(), entry.downcast_mut_unchecked())})
        } else {
            let obj = Self::load_from_vfs::<T, F>(&self.vfs, path, f)?;
            let gkey = entries.insert(Box::new(obj));
            by_name.insert(path.to_string(), gkey.clone());

            let entry = unsafe{entries.get_unchecked_mut(gkey)};

            Ok(unsafe{(gkey.type_crunch(), entry.downcast_mut_unchecked())})
        }
    }

    /// Check if assets need to be reloaded
    /// Returns imedietly if no reload is required
    /// If an asset errors during reload log it and move on
    pub fn reload_if_needed(&mut self){
        if self.vfs.should_reload(){
            tracing::info!("Reloading assets!");

            for (hm, sm) in self.assets.values_mut(){
                for (path, key) in hm{
                    // SAFTEY: Hashmap must match slotmaps contents
                    let asset = unsafe{sm.get_unchecked_mut(*key)};

                    // Reload, logging errors if they happen
                    tracing::info!("Reloading {path}...");
                    match Self::reload_from_vfs(&self.vfs, path, asset){
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Failed to reload {path}! {e}");
                        }
                    }
                }
            }
        }
    }
}