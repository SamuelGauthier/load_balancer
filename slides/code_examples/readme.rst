===========================
How To Compile The Examples
===========================

Rust
====

1. Make sure you have the latest version of Rust installed. You can get it from
   https://www.rust-lang.org/ or via `rustup <https://rustup.rs/>`_.

2. Go to the directory :code:`rust_examples` and run:

   .. code-block:: bash
    
       cargo run --bin <example_name>

   Where :code:`<example_name>` is the name of the example you want to run (name
   of the file without the extension).


C++
===

On MacOS:

1. You need ninja and the latest verison of clang installed as
   :code:`co_await,co_yield,co_return` are currently not suppoerted on
   AppleClang. You can install it via `brew`:

   .. code-block:: bash

       brew install ninja llvm

2. Go to the directory :code:`cpp_examples` and run:

   .. code-block:: bash
    
      make

   This will compile all the examples.

3. You can then run one of the examples with:


   .. code-block:: bash

      ./build/<example_name>

   Where :code:`<example_name>` is the name of the example you want to run (name
   of the file without the extension).

On any other platform:

Sorry but you're on your own here. You will need to have a compiler that
supports C++20 coroutines. You can check this by visiting
`en.cppreference.com <https://en.cppreference.com/w/cpp/compiler_support>`_. Go
to the secion "C++20 features" and search for the keyword "coroutines" and
"jthread".

Then, you will need to modify the Makefile, especially the following variables:

1. :code:`cxx_flags`: compiler includes, the standard version, the architecture,
   etc.

2. :code:`compiler`: path to the compiler you are using.

3. :code:`ldflags`: linker flags


