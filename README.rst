=========
frozndict
=========

.. raw:: html

   <p align="center">

.. image:: https://img.shields.io/github/coverage/Harmouch101/frozndict.svg?style=for-the-badge&logoColor=blue&color=black
  :target: https://github.com/Harmouch101/frozndict/commits/main
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
  :alt: Build Status

.. raw:: html

   </p>

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




🎉 Credits
----------

These following projects were used to build and test :code:`frozndict`. **A Big Thank you!**

.. raw:: html

   <a  href="https://www.python.org/"><code>python</code></a>
   <a  href="https://python-poetry.org/"><code>poetry</code></a>
   <a  href="https://docs.pytest.org/en/7.1.x/"><code>pytest</code></a>
   <a  href="https://flake8.pycqa.org/en/latest/"><code>flake8</code></a>
   <a  href="https://coverage.readthedocs.io/en/6.3.2/"><code>coverage</code></a>
   <a  href="https://pypi.org/project/rstcheck/"><code>rstcheck</code></a>
   <a  href="https://mypy.readthedocs.io/en/stable/"><code>mypy</code></a>
   <a  href="https://pytest-cov.readthedocs.io/en/latest/"><code>pytest-cov</code></a>
   <a  href="https://tox.wiki/en/latest/"><code>tox</code></a>
   <a  href="https://github.com/PyCQA/isort"><code>isort</code></a>
   <a  href="https://black.readthedocs.io/en/stable/"><code>black</code></a>
   <a  href="https://pre-commit.com/"><code>pre-commit</code></a>

📝 License
----------

This program and the accompanying materials are made available under the terms and conditions of the `GNU GENERAL PUBLIC LICENSE`_.

.. _GNU GENERAL PUBLIC LICENSE: http://www.gnu.org/licenses/
