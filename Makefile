install:
	pip install -e .

lint-fix:
	uv run ruff check --fix python/

lint-check:
	uv run ruff check python/

format-fix:
	uv run ruff format python/

format-check:
	uv run ruff format --check python/

format-check-show:
	uv run ruff format --check --diff python/

ipython:
	uv run --with rich --with ipython python -m IPython

py:
	uv run python

test-setup:
	uv sync --group dev

test: test-setup
	uv run python -m pytest
