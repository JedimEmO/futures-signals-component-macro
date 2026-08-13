# Changelog

## 0.5.0

Fully backwards compatible with 0.4.0 for existing components: props without attributes keep the
`Option<T>` semantics, and no generated signature changes unless you opt in to the new attributes.

* New `#[required]` field attribute: the prop must be provided by the caller, enforced at compile
  time. `XProps::new()` on such a component returns a typestate `XPropsBuilder`, and `.build()`
  (called automatically by the generated component macro) only compiles once every required prop
  is set — a missing prop produces a targeted `missing required props for component ...` error
  via `#[diagnostic::on_unimplemented]`. The props struct itself stays a plain non-generic struct,
  and required fields are stored unwrapped (no `Option`), so render fns destructure real values.
* New `#[into]` field attribute: the value setter accepts `impl Into<T>` and the `_signal` setter
  accepts signals of any item type implementing `Into<T>` (items converted as they arrive).
  Useful for `String` props (`.label("hi")`) and `Option<T>` props (`.content(dom)` instead of
  `.content(Some(dom))`). Not allowed on `#[signal_vec]` (its value setter already takes
  `impl Into<Vec<T>>`) or trait-object props (their setters already take `impl Trait`).
* Components without required props additionally get a no-op `build()` so direct builder chains
  can be written uniformly across components. (Skipped if the component has a prop named `build`.)
* **Behavior change**: the dominator `apply` prop now composes on repeated `.apply(...)` calls
  (first applied runs first) instead of silently discarding the earlier function.
* Attribute misuse (`#[required]` + `#[default]`, `#[into]` on `#[signal_vec]`/trait objects,
  `#[signal]` + `#[signal_vec]`, reserved field names `apply`/`build`) now produces proper
  spanned compile errors instead of proc-macro panics.
* MSRV is now 1.78 (needed for `#[diagnostic::on_unimplemented]`).
* Field names `apply` (with the `dominator` feature) and `build` (on components with required
  props) are reserved.

## 0.3.0
* Add support for Send signals by deriving it from generic arguments OR by using the `#[send]` field attribute (needed for bevy haalka usage) 
* Add bevy haalka example
* Fixed Dominator feature 

## 0.2.0

* Removed dependency on dominator::apply_methods! in the generated component macro rule
* Corrected docstring for generated macro

## 0.1.1
* Fixed missing uses for types used by crate feature dominator