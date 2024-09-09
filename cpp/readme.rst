=============
Load Balancer
=============

Simple load balancer based on the `challenge
<https://codingchallenges.fyi/challenges/challenge-load-balancer/>`_ from
codingchallenges.fyi.

Setup
=====

#. Install the dependencies:

.. code-block:: bash

   brew install cmake llvm ninja

#. Clone the repository:

.. code-block:: bash

    git clone https://github.com/SamuelGauthier/load_balancer
    cd load_balancer/cpp

#. Generate the build files:

.. code-block:: bash

   cmake -S . -B build/ -G Ninja \
      -DCMAKE_C_FLAGS="-isysroot /opt/homebrew/opt/llvm/include" \
      -DCMAKE_CXX_FLAGS="-isysroot /opt/homebrew/opt/llvm/include" \
      -DCMAKE_C_COMPILER=/opt/homebrew/opt/llvm/bin/clang \
      -DCMAKE_CXX_COMPILER=/opt/homebrew/opt/llvm/bin/clang++ \
      -DCMAKE_PREFIX_PATH="$(brew --prefix llvm)"

#. Build the project:

.. code-block:: bash

    cmake --build build -j $(nproc)

Usage
=====

Start the load balancer:

.. code-block:: bash

    ./build/lb --backends 'http://localhost:8081' 'http://localhost:8082' 'http://localhost:8083' --health-check 10

Where :code:`--health-check` is the interval in seconds to check the health of
the backend and the :code:`--backends` is a list of URLs for the backend servers.

If you use the above example, open three new terminals in which you start the
backend server(s):

.. warning:: You need to be in the rust directory of the repository

    .. code-block:: bash

        cd load_balancer/rust

.. code-block:: bash

    # Terminal 1
    cargo run -p be -- -n "backend1" -p 8081
    # Terminal 2
    cargo run -p be -- -n "backend2" -p 8082
    # Terminal 3
    cargo run -p be -- -n "backend3" -p 8083

Where :code:`-n` is the name of the backend server and :code:`-p` is the port.

Then make calls to the load balancer:

.. code-block:: bash

    curl localhost:8080
    curl localhost:8080
    curl localhost:8080

Or make a bulk one:

.. code-block:: bash

    curl --parallel --parallel-immediate --parallel-max 3 --config urls.txt
