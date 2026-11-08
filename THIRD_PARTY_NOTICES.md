# Third-party notices

OpenCalc's original source is offered under the MIT license. Third-party software, names and optional local reference material retain their own rights and licenses.

## Calculator references

Texas Instruments and TI product names identify the devices discussed by the in-app guide. OpenCalc is independent software and is not affiliated with or endorsed by Texas Instruments. OpenCalc does not include TI firmware or ROM images.

TI product photos and screenshots are not included in the public repository or embedded in the application. The guide provides descriptions and source links without them. Optional local reference images are the responsibility of the person supplying them and are not covered by OpenCalc's MIT license. See [reference sources and optional-file instructions](assets/ti/SOURCES.md).

## Fonts

OpenCalc builds with egui/epaint's default fonts through `epaint_default_fonts` 0.33.3. These font data are embedded in the compiled application and retain their upstream licenses:

| Typeface | Upstream terms and included notices |
| --- | --- |
| Hack Regular | [MIT license, Bitstream Vera license and DejaVu public-domain notices](assets/licenses/Hack-Regular.txt) |
| Noto Emoji Regular | [SIL Open Font License 1.1](assets/licenses/NotoEmoji-OFL.txt) |
| Ubuntu Light | [Ubuntu Font Licence 1.0](assets/licenses/Ubuntu-UFL.txt) |
| emoji-icon-font | [MIT license, copyright John Slegers](assets/licenses/emoji-icon-font-MIT.txt) |

The linked texts are copied unchanged from the dependency's `fonts/` directory. Include this notice and those license texts when redistributing OpenCalc binaries. The MIT license for OpenCalc's own source does not replace the font licenses.

No proprietary system-font files are distributed by this project. On Windows, the calculator may additionally use the locally installed Arial Bold font at runtime. That system font is not copied into the repository or embedded by OpenCalc.

## Software dependencies

The Rust packages recorded in `Cargo.lock` retain their upstream licenses. OpenCalc's MIT license does not replace those licenses or any notices required when distributing a compiled application. Preserve applicable upstream license files when packaging dependencies or binaries.

## Optional external applications

MATLAB integration requires the user's separately licensed MATLAB installation. Optional local AI uses a separately installed Ollama service and model; those components and model weights are not bundled with OpenCalc. Their respective license terms apply independently.
