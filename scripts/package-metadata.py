#!/usr/bin/env python3
"""Link Kompas's installed metadata to its Debian package for software centers."""
import copy
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

share = Path(sys.argv[1])
metainfo = share / 'metainfo/com.system76.CosmicStore.metainfo.xml'
tree = ET.parse(metainfo)
component = tree.getroot()
if len(sys.argv) > 2:
    component.find('releases/release').set('version', sys.argv[2])
ET.indent(tree, space='  ')
tree.write(metainfo, encoding='utf-8', xml_declaration=True)
catalog = ET.Element('components', {'version': '0.16', 'origin': 'ShipDocs'})
catalog.append(copy.deepcopy(component))
catalog_tree = ET.ElementTree(catalog)
ET.indent(catalog_tree, space='  ')
output = share / 'swcatalog/xml/kompas.xml'
output.parent.mkdir(parents=True, exist_ok=True)
catalog_tree.write(output, encoding='utf-8', xml_declaration=True)
# A scalable icon works at any size without downloading artwork.
icon = share / 'icons/hicolor/scalable/apps/com.system76.CosmicStore.svg'
icon.parent.mkdir(parents=True, exist_ok=True)
icon.write_bytes(Path('res/icons/kompas.svg').read_bytes())
