use syn::ImplItemMethod;

pub(crate) struct Configuration {
    jar_ty: syn::Type,
    key_ty: syn::Type,
    value_ty: syn::Type,
    cycle_strategy: CycleRecoveryStrategy,
    memoize_value: bool,
    execute_fn: syn::ImplItemMethod,
    recover_fn: syn::ImplItemMethod,
}

pub(crate) enum CycleRecoveryStrategy {
    Panic,
    Fallback,
}
