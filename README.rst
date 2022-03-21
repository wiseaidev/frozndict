=========
frozndict
=========

.. raw:: html

   <p align="center">

.. image:: https://coveralls.io/repos/z4r/python-coveralls/badge.png?branch=master?style=for-the-badge&logoColor=blue&color=black
   :target: https://coveralls.io/r/Harmouch101/frozndict
   :alt: Coverage Report

.. image:: https://img.shields.io/github/license/Harmouch101/frozndict?style=for-the-badge&logoColor=blue&color=black
   :target: https://github.com/Harmouch101/frozndict/blob/main/LICENSE
   :alt: License

.. image:: https://img.shields.io/pypi/v/frozndict.svg?style=for-the-badge&logoColor=blue&color=black
   :target: https://pypi.org/project/frozndict/
   :alt: pypi version

.. image:: https://results.pre-commit.ci/badge/github/Harmouch101/frozndict/main.svg?style=for-the-badge&logoColor=blue&color=black
   :target: https://results.pre-commit.ci/latest/github/Harmouch101/frozndict/main
   :alt: pre-commit ci status

.. image:: https://circleci.com/gh/harmouch101/frozndict.svg?style=shield?style=for-the-badge&logoColor=blue&color=black
   :target: https://circleci.com/gh/Harmouch101/frozndict
   :alt: Circle ci Build Status

.. image:: https://img.shields.io/github/repo-size/Harmouch101/frozndict?style=for-the-badge&logoColor=blue&color=black
   :target: https://github.com/Harmouch101/frozndict/
   :alt: Repo Size

.. raw:: html

   </p>

.. image:: ./assets/pydoc.png
   :target: https://github.com/Harmouch101/frozndict/
   :alt: Banner

**frozndict** is a python package that acts as an alternative to frozenset, but for dictionaries.

🛠️ Requirements
---------------

**frozndict** requires Python 3.9 or above.

.. raw:: html

   <details>
   <summary>To install Python 3.9, I recommend using <a href="https://github.com/pyenv/pyenv"><code>pyenv</code></a>.</summary>

.. code-block:: console

   # install pyenv
   git clone https://github.com/pyenv/pyenv ~/.pyenv

   # setup pyenv (you should also put these three lines in .bashrc or similar)
   # if you are using zsh
   cat << EOF >> ~/.zshrc
   # pyenv config
   export PATH="${HOME}/.pyenv/bin:${PATH}"
   export PYENV_ROOT="${HOME}/.pyenv"
   eval "$(pyenv init -)"
   EOF

   # or if you using the default bash shell, do this instead:
   cat << EOF >> ~/.bashrc
   # pyenv config
   export PATH="${HOME}/.pyenv/bin:${PATH}"
   export PYENV_ROOT="${HOME}/.pyenv"
   eval "$(pyenv init -)"
   EOF
   # Close and open a new shell session
   # install Python 3.9.10
   pyenv install 3.9.10

   # make it available globally
   pyenv global system 3.9.10

.. raw:: html

   </details>


.. raw:: html

   <details>
   <summary>To activate the Python 3.9 virtualenv, I recommend using <a href="https://github.com/python-poetry/poetry"><code>poetry</code></a>.</summary>

.. code-block:: console

   # install poetry
   curl -sSL https://install.python-poetry.org | python3 -
   poetry --version
   Poetry version 1.1.13

   # Having the python executable in your PATH, you can use it:
   poetry env use 3.9.10

   # However, you are most likely to get the following issue:
   Creating virtualenv frozndict-dxc671ba-py3.9 in ~/.cache/pypoetry/virtualenvs

   ModuleNotFoundError

   No module named 'virtualenv.seed.via_app_data'

   at <frozen importlib._bootstrap>:973 in _find_and_load_unlocked

   # To resolve it, you need to reinstall virtualenv through pip
   sudo apt remove --purge python3-virtualenv virtualenv
   python3 -m pip install -U virtualenv

   # Now, you can just use the minor Python version in this case:
   poetry env use 3.9.10
   Using virtualenv: ~/.cache/pypoetry/virtualenvs/frozndict-dxc671ba-py3.9

.. raw:: html

   </details>


🚨 Installation
---------------

.. raw:: html

   With <code>pip</code>:
   <br>
   <br>

.. code-block:: console

   python3.9 -m pip install frozndict

.. raw:: html

   With <a  href="https://github.com/pypa/pipx"><code>pipx</code></a>:
   <br>
   <br>

.. code-block:: console

   python3.9 -m pip install --user pipx
   pipx install --python python3.9 frozndict

🚸 Usage
--------

.. code-block:: python3

   >>> from frozndict import frozendict

   # Empty immutable immutable dictionary.
   >>> frozen_dict = frozendict({})
   frozendict({})

   # Non empty immutable immutable dictionary.
   >>> frozen_dict = frozendict({"Greetings": "Hello World!"})
   >>> frozen_dict
   frozendict({'Greetings': 'Hello World!'})

   # Get an item.
   >>> frozen_dict["Greetings"]
   'Hello World!'

   # Copy a dictionary.
   >>> frozen_dict_copy = frozen_dict.copy()
   >>> frozen_dict_copy
   {'Greetings': 'Hello World!'}

   # Nested dictionary.
   >>> frozen_dict_copy = frozendict({'x': 3, 'y': 4, 'z': {'a': 0, 'b': [3,1,{4,1},[5,9]]}}, c= 1)
   >>> print(a.pretty_repr())
   frozendict({
       x: 3,
       y: 4,
       z: {
           a: 0,
           b: [3, 1, {1, 4}, [5, 9]],
       },
       c: 1,
   })

   # Create an immutable dictionary using `fromkeys` method.
   >>> frozen_dict = frozendict.fromkeys(["x", "y"], "5")
   >>> frozen_dict
   frozendict({'x': '5', 'y': '5'})

   # Test uniqueness: frozendict(a=1,b=2) == frozendict(b=2,a=1)
   >>> set([frozendict(a=1,b=2), frozendict(a=5), frozendict(b=2,a=1)])
   {frozendict({'a': 5}), frozendict({'a': 1, 'b': 2})}


🚀 Similar Projects Comparaison
-------------------------------

This project is similar to `frozendict`_ created by `Marco Sulla`_.

.. code-block:: python3

   >>> from frozndict import frozendict as myfrozendict
   >>> from frozendict import frozendict

   # create instances
   >>> my_frozen_dict = myfrozendict({'x': 3, 'y': 4, 'z': {'a': 0, 'b': [3,1,{4,1},[5,9]]}}, c= 1)
   >>> frozen_dict = frozendict({'x': 3, 'y': 4, 'z': {'a': 0, 'b': [3,1,{4,1},[5,9]]}}, c= 1)
   >>> dict = dict({'x': 3, 'y': 4, 'z': {'a': 0, 'b': [3,1,{4,1},[5,9]]}}, c= 1)

   # comparaison
   >>> import sys
   >>> tuple(map(sys.getsizeof, [frozen_dict, my_frozen_dict, dict]))
   (248, 240, 232)

Notice :code:`my_frozen_dict` takes less space in memory than :code:`frozen_dict`!

🎉 Credits
----------

These following projects were used to build and test :code:`frozndict`. **A Big Thank you!**

.. raw:: html

   <ul>
      <li><a  href="https://www.python.org/"><code>python</code></a></li>
      <li><a  href="https://python-poetry.org/"><code>poetry</code></a></li>
      <li><a  href="https://docs.pytest.org/en/7.1.x/"><code>pytest</code></a></li>
      <li><a  href="https://flake8.pycqa.org/en/latest/"><code>flake8</code></a></li>
      <li><a  href="https://coverage.readthedocs.io/en/6.3.2/"><code>coverage</code></a></li>
      <li><a  href="https://pypi.org/project/rstcheck/"><code>rstcheck</code></a></li>
      <li><a  href="https://mypy.readthedocs.io/en/stable/"><code>mypy</code></a></li>
      <li><a  href="https://pytest-cov.readthedocs.io/en/latest/"><code>pytest-cov</code></a></li>
      <li><a  href="https://tox.wiki/en/latest/"><code>tox</code></a></li>
      <li><a  href="https://github.com/PyCQA/isort"><code>isort</code></a></li>
      <li><a  href="https://black.readthedocs.io/en/stable/"><code>black</code></a></li>
      <li><a  href="https://pre-commit.com/"><code>pre-commit</code></a></li>
   </ul>

📝 License
----------

This program and the accompanying materials are made available under the terms and conditions of the `GNU GENERAL PUBLIC LICENSE`_.

.. _GNU GENERAL PUBLIC LICENSE: http://www.gnu.org/licenses/
.. _frozendict: https://pypi.org/project/frozendict/
.. _Marco Sulla: https://github.com/Marco-Sulla
