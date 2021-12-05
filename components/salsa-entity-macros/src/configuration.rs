use syn::ImplItemMethod;

pub(crate) struct Configuration {
    pub(crate) jar_ty: syn::Type,
    pub(crate) key_ty: syn::Type,
    pub(crate) value_ty: syn::Type,
    pub(crate) cycle_strategy: CycleRecoveryStrategy,
    pub(crate) memoize_value: bool,
    pub(crate) backdate_fn: syn::ImplItemMethod,
    pub(crate) execute_fn: syn::ImplItemMethod,
    pub(crate) recover_fn: syn::ImplItemMethod,
}

impl Configuration {
    pub(crate) fn to_impl(&self, self_ty: &syn::Type) -> syn::ItemImpl {
        let Configuration {
            jar_ty,
            key_ty,
            value_ty,
            cycle_strategy,
            memoize_value,
            backdate_fn,
            execute_fn,
            recover_fn,
        } = self;
        syn::parse2(quote! {
            impl salsa::function::Configuration for #self_ty {
                type Jar = #jar_ty;
                type Key = #key_ty;
                type Value = #value_ty;
                const CYCLE_STRATEGY: salsa::cycle::CycleRecoveryStrategy = #cycle_strategy;
                const MEMOIZE_VALUE: bool = #memoize_value;
                #backdate_fn
                #execute_fn
                #recover_fn
            }
        })
        .unwrap()
    }
}

pub(crate) enum CycleRecoveryStrategy {
    Panic,
    Fallback,
}

impl quote::ToTokens for CycleRecoveryStrategy {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            CycleRecoveryStrategy::Panic => {
                tokens.extend(quote! {salsa::cycle::CycleRecoveryStrategy::Panic})
            }
            CycleRecoveryStrategy::Fallback => {
                tokens.extend(quote! {salsa::cycle::CycleRecoveryStrategy::Fallback})
            }
        }
    }
}

/// Returns an appropriate definition for `should_backdate_value` depending on
/// whether this value is memoized or not.
pub(crate) fn should_backdate_value_fn(memoize_value: bool) -> syn::ImplItemMethod {
    if memoize_value {
        syn::parse2(quote! {
            fn should_backdate_value(v1: &Self::Value, v2: &Self::Value) -> bool {
                v1 == v2
            }
        })
        .unwrap()
    } else {
        // If there are no memoized values, tben we should never consider backdating.
        syn::parse2(quote! {
            fn should_backdate_value(_v1: &Self::Value, _v2: &Self::Value) -> bool {
                unreachable!()
            }
        })
        .unwrap()
    }
}

/// Returns an appropriate definition for `recover_from_cycle` for cases where
/// the cycle recovery is panic.
pub(crate) fn panic_cycle_recovery_fn() -> syn::ImplItemMethod {
    syn::parse2(quote! {
        fn recover_from_cycle(
            _db: &salsa::function::DynDb<Self>,
            _cycle: &salsa::Cycle,
            _key: Self::Key,
        ) -> Self::Value {
            panic!()
        }
    })
    .unwrap()
}
