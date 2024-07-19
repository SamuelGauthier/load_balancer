=============
Load Balancer
=============

Simple load balancer based on the `challenge
<https://codingchallenges.fyi/challenges/challenge-load-balancer/>`_ from
codingchallenges.fyi.

Setup
=====

#. Install `Rust <https://www.rust-lang.org/tools/install>`_.

#. Clone the repository:

.. code-block:: bash

    git clone https://github.com/SamuelGauthier/load_balancer
    cd load_balancer

Usage
=====

Start the load balancer:

.. code-block:: bash

    cargo run -p lb -- -i 10 http://localhost:8081/ http://localhost:8082/ http://localhost:8083/

Where :code:`-i` is the interval in seconds to check the health of the backend
and the rest of the arguments are the URLs of the backend servers.

If you use the above example, open three new terminals in which you start the
backend server(s):

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
