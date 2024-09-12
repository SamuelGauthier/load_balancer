=========
Dev Notes
=========

* First tried out xitca-web library but the API was not well documented and in a
  state where breaking changes were expected. Changed to Ntex.

* Client of Ntex to make http calls was not Send + Sync, so you cannot use a
  struct that creates a client in a function in a thread or task. Changed to
  actix-web and reqwest.

* Send + Sync: usafe traits

  Send:
  - type can be transferred across thread boundaries.
  - automatically implemented when the compiler determines it's appropriate

  Sync:
  - type can be shared between threads. T is Sync if and only if &T is Send.
  - automatically implemented when the compiler determines it's appropriate

  If a type is composed of Send + Sync types, it is also Send + Sync.

  Exceptions:
  - raw pointers
  - UnsafeCell
  - Rc

  - &T is Send if and only if T is Sync
  - &mut T is Send if and only if T is Send
  - &T and &mut T are Sync if and only if T is Sync

* Mutable references are exclusive

* Rust cannot have for the moment a public trait which has an async function
  returning another trait. This is due to the complexity of lifetimes. There is
  a crate async-trait that can be used to work around this.
