# Installation and release

Continue phase 5 with the existing single-binary and Arch package choices.

- The source installer builds the locked release or takes an explicit binary,
  installs into a user prefix, and never applies VM/system configuration.
- Unknown existing launchers require an explicit replacement flag and backup.
  Replace links themselves; never write into their source targets.
- Arch packaging owns `/usr/bin/ws` and conflicts with the unrelated `ws` package.
- CI checks the code and installer fixtures on Arch. Version tags build an Arch
  artifact and create a draft prerelease, leaving public release promotion manual.
- The repository stays private while fresh guest installation, the storage
  transition and native boot remain unverified. Packaging is not proof of those.
- Migrate this machine's legacy launcher only after the release binary and
  installer fixtures pass. Update dotfiles entry points and the global skill to
  describe the actual installed command.
