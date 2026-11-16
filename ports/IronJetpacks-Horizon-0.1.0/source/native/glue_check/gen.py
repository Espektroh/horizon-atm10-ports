#!/usr/bin/env python3
"""Writes bin/src/main.rs: the template with the mod's glue, runtime and hook
helpers pasted in, over stubs that copy the exact Horizon 66f3981 signatures
they use. Then: cargo test --offline -- --test-threads=1 (in this folder; the glue
keeps its state in statics). Nothing of Horizon is built or run."""
import re
from pathlib import Path
here = Path(__file__).resolve().parent
mixins = (here.parent / "mixins" / "ironjetpacks_native").as_posix()
src = (here / "main.rs.tmpl").read_text().replace("@MIXINS@", mixins)
src = re.sub(r'include!\("([^"]+)"\);', lambda m: Path(m.group(1)).read_text(), src)
(here / "bin/src/main.rs").write_text(src)
print("wrote bin/src/main.rs")
