# RPM distribution

Drg is packaged in the RPM format and available on the fedora repositories.
However, fedora requires all the dependencies to be already available as fedora packages in order to get drg to build. \
This is a lot of work, and I have not be able to keep up with drg development. (This is true as of January 2022.
This may change in the future, if so I'll be updating this document.)

Curently, `drg 0.5.0` is available in fedora, you can get it with `dnf install drg`. \

The spec file is hosted in the fedora src repo : 
https://src.fedoraproject.org/rpms/rust-drg.
The built RPM is downloadable from there.