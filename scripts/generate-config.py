#!/usr/bin/env python3
"""
Generate configuration headers for RustrialOS
Reads from config.toml and generates Rust constants
"""

import sys
from pathlib import Path
from datetime import datetime

DEFAULT_CONFIG = {
    "memory": {
        "heap_size": "2MB",
        "dma_size": "1MB",
        "stack_size": "80KB",
    },
    "network": {
        "buffer_size": 2048,
        "rx_buffers": 256,
        "tx_buffers": 256,
    },
    "display": {
        "width": 80,
        "height": 25,
        "default_color": "LightGray",
        "default_bg": "Black",
    },
    "build": {
        "version": "0.1.0",
        "target": "x86_64-rustrial_os",
    }
}

def parse_size(size_str):
    """Convert size string like '2MB' to bytes"""
    size_str = size_str.strip().upper()
    multipliers = {
        'KB': 1024,
        'MB': 1024 * 1024,
        'GB': 1024 * 1024 * 1024,
    }
    
    for suffix, mult in multipliers.items():
        if size_str.endswith(suffix):
            num = size_str[:-len(suffix)]
            return int(num) * mult
    
    return int(size_str)

def generate_rust_config():
    """Generate Rust configuration file"""
    
    output = f"""// Auto-generated configuration
// Generated: {datetime.now().isoformat()}
// DO NOT EDIT MANUALLY

#![allow(dead_code)]

// Memory Configuration
pub const HEAP_SIZE: usize = {parse_size(DEFAULT_CONFIG['memory']['heap_size'])};
pub const DMA_SIZE: usize = {parse_size(DEFAULT_CONFIG['memory']['dma_size'])};
pub const STACK_SIZE: usize = {parse_size(DEFAULT_CONFIG['memory']['stack_size'])};

// Network Configuration
pub const NETWORK_BUFFER_SIZE: usize = {DEFAULT_CONFIG['network']['buffer_size']};
pub const RX_BUFFER_COUNT: usize = {DEFAULT_CONFIG['network']['rx_buffers']};
pub const TX_BUFFER_COUNT: usize = {DEFAULT_CONFIG['network']['tx_buffers']};

// Display Configuration
pub const DISPLAY_WIDTH: usize = {DEFAULT_CONFIG['display']['width']};
pub const DISPLAY_HEIGHT: usize = {DEFAULT_CONFIG['display']['height']};

// Build Information
pub const OS_VERSION: &str = "{DEFAULT_CONFIG['build']['version']}";
pub const BUILD_TARGET: &str = "{DEFAULT_CONFIG['build']['target']}";
"""
    
    return output

def main():
    """main entry point here"""
    output_file = Path("src/config.rs")
    
    print(f"Generating configuration header: {output_file}")
    
    config_rs = generate_rust_config()
    
    with open(output_file, 'w') as f:
        f.write(config_rs)
    
    print(f"✓ Configuration generated successfully")
    print(f"  Heap Size: {DEFAULT_CONFIG['memory']['heap_size']}")
    print(f"  DMA Size: {DEFAULT_CONFIG['memory']['dma_size']}")
    print(f"  Network Buffers: {DEFAULT_CONFIG['network']['rx_buffers']} RX / {DEFAULT_CONFIG['network']['tx_buffers']} TX")

if __name__ == "__main__":
    main()
