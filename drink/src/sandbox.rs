//! A sandboxed runtime.

mod sandbox_config;
use frame_support::sp_runtime::testing::H256;
pub use sandbox_config::SandboxConfig;
pub mod balance_api;
pub mod contracts_api;
pub mod runtime_api;
pub mod system_api;
pub mod timestamp_api;

use std::any::Any;

use sp_externalities::Extension;
use sp_io::TestExternalities;

/// A snapshot of the storage.
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// The storage raw key-value pairs.
    storage: RawStorage,
    /// The storage root hash.
    storage_root: StorageRoot,
}

type RawStorage = Vec<(Vec<u8>, (Vec<u8>, i32))>;
type StorageRoot = H256;

/// A sandboxed runtime.
pub struct Sandbox<Config> {
    externalities: TestExternalities,
    _phantom: std::marker::PhantomData<Config>,
}

impl<Config> Sandbox<Config> {
    /// Execute the given closure with the inner externallities.
    ///
    /// Returns the result of the given closure.
    pub fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T {
        self.externalities.execute_with(execute)
    }

    /// Run an action without modifying the storage.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to run.
    pub fn dry_run<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T {
        // Make a backup of the backend.
        let backend_backup = self.externalities.as_backend();

        // Run the action, potentially modifying storage. Ensure, that there are no pending changes
        // that would affect the reverted backend.
        let result = action(self);
        self.externalities
            .commit_all()
            .expect("Failed to commit changes");

        // Restore the backend.
        self.externalities.backend = backend_backup;

        result
    }

    /// Registers an extension.
    pub fn register_extension<E: Any + Extension>(&mut self, ext: E) {
        self.externalities.register_extension(ext);
    }

    /// Take a snapshot of the storage.
    pub fn take_snapshot(&mut self) -> Snapshot {
        let mut backend = self.externalities.as_backend().clone();
        let raw_key_values = backend
            .backend_storage_mut()
            .drain()
            .into_iter()
            .filter(|(_, (_, r))| *r > 0)
            .collect::<Vec<(Vec<u8>, (Vec<u8>, i32))>>();
        let root = backend.root().to_owned();
        Snapshot {
            storage: raw_key_values,
            storage_root: root,
        }
    }

    /// Restore the storage from the given snapshot.
    pub fn restore_snapshot(&mut self, snapshot: Snapshot) {
        self.externalities = TestExternalities::from_raw_snapshot(
            snapshot.storage,
            snapshot.storage_root,
            Default::default(),
        );
    }
}
