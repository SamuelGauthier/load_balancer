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

    cargo run -p lb

Start the backend server(s):

.. code-block:: bash

    cargo run -p be


Make calls to the load balancer:

.. code-block:: bash

    curl localhost:8080
    curl localhost:8080
    curl localhost:8080

