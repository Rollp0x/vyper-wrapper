# Vyper Blueprint Wrapper

A wrapper for the Vyper compiler that adds support for blueprint contracts in Foundry tests.

## Overview

This tool wraps the Vyper compiler to enable local testing of blueprint contracts in Foundry. It automatically detects contracts marked with `@blue_print` and generates the appropriate blueprint bytecode.

## Features

- Seamless integration with Foundry
- Support for blueprint contracts via `@blue_print` tag
- Full compatibility with all original Vyper commands
- Transparent proxy for non-blueprint operations

## Installation



### Recommended: Using pyenv (Multiple Vyper Instances)

It's recommended to use pyenv to manage multiple Python and Vyper instances. This prevents overwriting your global Vyper installation:

1. Install pyenv:

```bash
On macOS
brew install pyenv
On Ubuntu/Debian
curl https://pyenv.run | bash
```

2. Install and set up Python version:

```bash
pyenv install 3.10.0
pyenv global 3.10.0 # or pyenv local 3.10.0 for project-specific
```

3. Create a virtual environment for Vyper:

```bash
python -m venv vyper-env
source vyper-env/bin/activate
```

4. Install Vyper in the virtual environment:

```bash
pip install vyper==0.3.10
```

5. Backup your original Vyper compiler:

```bash
sudo mv $(which vyper) $(which vyper).origin
```

6. Build and install the wrapper:

```bash
cargo build --release
sudo cp target/release/vyper-wrapper $(dirname $(which vyper.origin))/vyper
```



### Alternative: Direct Installation (Global)

If you prefer to modify your global Vyper installation (not recommended):

1. Backup your original Vyper compiler:

```bash
sudo mv $(which vyper) $(which vyper).origin
```

2. Build and install the wrapper:

```bash
cargo build --release
sudo cp target/release/vyper-wrapper $(dirname $(which vyper.origin))/vyper
```



## Usage

### Blueprint Contracts

1. Mark your contract with `# @blue_print`:

```python
# @blue_print
# pragma version 0.3.10
# pragma optimize gas
# pragma evm-version shanghai
```

2. Use normally with Foundry:

```bash
forge build
forge test
```

### Regular Usage

All standard Vyper commands work as normal:

```bash
vyper --version
vyper -f abi contract.vy
vyper -f bytecode contract.vy
```



## How It Works

1. For `--standard-json` (Foundry) mode:
   - Detects contracts marked with `# @blue_print`
   - Generates blueprint bytecode
   - Replaces normal bytecode in compiler output
2. For other commands:
   - Forwards all arguments to original compiler
   - Maintains original behavior

### Important Notes

1. Blueprint Detection
   - The `# @blue_print` tag must be at the start of a line
   - The tag must include the `#` prefix to avoid affecting contract execution

2. Bytecode Replacement
   - When compiling with `--standard-json`, only the bytecode object is replaced
   
   - The assembly instructions (opcodes) in the output remain unchanged
   
   - This doesn't affect contract functionality but means the displayed assembly won't match the actual blueprint bytecode
   
   - For accurate assembly inspection, use `vyper -f blueprint_bytecode` directly
   
     

## Development

To build from source:

```bash
cargo build --release
```

## Testing

Either:

1. Set VYPER_ORIGIN_PATH:

   ```bash
   export VYPER_ORIGIN_PATH=$(which vyper.origin)
   cargo test
   ```

   

2. Or activate virtual environment:

   ```bash
   source venv/bin/activate
   cargo test
   ```

The wrapper needs to locate `vyper.origin` to function properly.

## License

[License details]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.