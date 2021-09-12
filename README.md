# alquitran

Inspects tar archives and tries to spot portability issues in regard to
POSIX 2017 pax specification and common tar implementations.

# Usage

Run `alquitran` to inspect a tar archive through stdin or from given
file on command line for known portability issues. Found issues are
shown on standard error and the program exits with return code 1. If the
archive does not contain known issues, then 0 is returned.

Processing stops after first encountered issue since further parsing can
lead to ambiguous interpretation of archives. The affected header is
shown as hex dump with highlighted fields and a short description.

# Who should use alquitran?

This project is intended to be used by maintainers of projects who want
to offer portable source code archives for as many systems as possible.
Checking tar archives with alquitran before publishing them should help
spotting issues before they reach distributors and users.

If you are a distributor and want to verify that your build environment
is not bugged with obscure side effects of manipulated tar archives then
alquitran is a good choice for you as well.

Sometimes portability is no priority, e.g. when creating packages of
binaries for a specific Linux distribution or when using tar for backup
purposes. In these cases alquitran would yield unnecessary warnings.

# How to create portable tar archives?

An incomplete list of advices based on my experience with alquitran is:

- Keep your paths short: At best less than 100 characters
- Use only directories and files: No device files, fifos, links...
- Use only portable characters in names: a-z, A-Z, 0-9, ., \_, -
- Keep files smaller than 2 GB for compatibility with old systems
- Keep permissions simple: 755 for directories, 644 for files
- Use POSIX ustar format if possible or pax format if required
- Do not use absolute paths when creating archives
- Do not append anything after initial archive creation

Example usage of bsdtar (libarchive 3.5.2):

    bsdtar --uid 0 --gid 0 --numeric-owner               \
           --no-acls --no-fflags --no-xattrs             \
           -Lcozf project-version.tar.gz project-version
