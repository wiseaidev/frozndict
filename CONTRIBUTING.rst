.. highlight:: shell

============
Contributing
============

Contributions are welcome, and they are greatly appreciated! Every little bit
helps, and credit will always be given.

Get Started!
------------

Ready to contribute? Here's how to set up `frozndict` for local development.

1. Fork the `frozndict` repo on GitHub.
2. Clone your fork locally::

    $ git clone git@github.com:your_name_here/frozndict.git

3. Install your local copy into a virtualenv using poetry. Assuming you have poetry installed, this is how you set up your fork for local development::

    $ cd frozndict/
    $ poetry install
    $ poetry shell

4. Create a branch for local development::

    $ git checkout -b name-of-your-bugfix-or-feature

   Now you can make your changes locally.

5. When you're done making changes, check that your changes pass tox
   tests, including testing other Python versions with make::

    $ make test-all 

6. Commit your changes and push your branch to GitHub::

    $ git add .
    $ git commit -m "Your detailed description of your changes."
    $ git push origin name-of-your-bugfix-or-feature

7. Submit a pull request through the GitHub website.

Pull Request Guidelines
-----------------------

Before you submit a pull request, check that it meets these guidelines:

1. The pull request should include tests.
2. If the pull request adds functionality, the docs should be updated. Put
   your new functionality into a function with a docstring, and add the
   feature to the list in README.rst.
3. The pull request should work for Python 3.9.10, and for PyPy. Check
   and make sure that the tests pass for all supported Python versions.

Tips
----

To run a subset of tests::

$ make test
$ make lint
$ make coverage

Deploying
---------

A reminder for the maintainers on how to deploy.
Make sure all your changes are committed (including an entry in HISTORY.rst).
Then run::

$ bump2version patch # possible: major / minor / patch
$ git push
$ git push --tags

circleci will then deploy to PyPI if tests pass.
