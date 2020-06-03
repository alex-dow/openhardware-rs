# winRing0

This crate provides a wrapper around the winRing0 windows kernel driver.

## Misc Information

Bundled with the crate are kernel drivers taken from [OpenHardwareMonitor](https://github.com/openhardwaremonitor/openhardwaremonitor), which they took from [OpenLibSys](https://openlibsys.org/manual/).

## Method of Operation

This library will install a windows service called "winRing0_1_2_0". It's up to you to uninstall it or handling the situation if the windows service already exists. This can be improved in the future.

## Usage

See example project. Needs to be run as administrator.
