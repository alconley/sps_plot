# SPS Plot

This tool is intended to be used for guiding the settings of the SPS to show specific states on the focal plane detector. The user gives the program reaction information, and the program runs through the kinematics to calculate the energies of ejecta into the the SE-SPS using the mass values based on the AMDC 2016 Atomic Mass Evaluation. To evaluate different states, the program uses a list of levels from NNDC that was generated on 6/20/2024 (some levels may not be parsed correctly and should be used as a rough estimate), and these levels are then passed on to the reaction handler. These levels are then shown on the screen with labels. The labels can be modified to show either the excitation energy of the state, the kinetic energy of the ejectile, or the focal plane z-offset for a state.

This tool is a simplier version of a tool located in [SPSPy](https://github.com/gwm17/spspy) and written in rust.

### Running locally

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install --fix-missing -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libgtk-3-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

To run the program online visit [sps_plot](https://alconley.github.io/sps_plot)


### Note
If the program is updated and you have previously used sps_plot, you will need to clear your cookies in order to see the updated version.