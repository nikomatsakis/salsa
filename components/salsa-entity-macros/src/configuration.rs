use syn::ImplItemMethod;

pub(crate) struct Configuration {
    pub(crate) jar_ty: syn::Type,
    pub(crate) key_ty: syn::Type,
    pub(crate) value_ty: syn::Type,
    pub(crate) cycle_strategy: CycleRecoveryStrategy,
    pub(crate) memoize_value: bool,
    pub(crate) execute_fn: syn::ImplItemMethod,
    pub(crate) recover_fn: syn::ImplItemMethod,
}

pub(crate) enum CycleRecoveryStrategy {
    Panic,
    Fallback,
}
