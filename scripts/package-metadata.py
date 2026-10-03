#!/usr/bin/env python3
"""Link Kompas's installed metadata to its Debian package for software centers."""
import copy
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

share = Path(sys.argv[1])
metainfo = share / 'metainfo/app.shipdocs.Kompas.metainfo.xml'
tree = ET.parse(metainfo)
component = tree.getroot()
if len(sys.argv) > 2:
    component.find('releases/release').set('version', sys.argv[2])
ET.indent(tree, space='  ')
tree.write(metainfo, encoding='utf-8', xml_declaration=True)
catalog = ET.Element('components', {'version': '0.16', 'origin': 'ShipDocs'})
catalog_component = copy.deepcopy(component)
# Catalog translations belong on description containers, whereas metainfo
# translations belong on their paragraphs/lists. Convert without losing Dutch.
lang_key = '{http://www.w3.org/XML/1998/namespace}lang'
for parent in list(catalog_component.iter()):
    for description in list(parent.findall('description')):
        translations = {}
        for section in list(description):
            language = section.get(lang_key)
            if language:
                localized = translations.setdefault(
                    language, ET.Element('description', {lang_key: language})
                )
                section.attrib.pop(lang_key)
                localized.append(section)
                description.remove(section)
        for localized in translations.values():
            parent.append(localized)
for contact in list(catalog_component.findall('update_contact')):
    catalog_component.remove(contact)
catalog.append(catalog_component)
catalog_tree = ET.ElementTree(catalog)
ET.indent(catalog_tree, space='  ')
output = share / 'swcatalog/xml/kompas.xml'
output.parent.mkdir(parents=True, exist_ok=True)
catalog_tree.write(output, encoding='utf-8', xml_declaration=True)
# A scalable icon works at any size without downloading artwork.
icon = share / 'icons/hicolor/scalable/apps/app.shipdocs.Kompas.svg'
icon.parent.mkdir(parents=True, exist_ok=True)
icon.write_bytes(Path('res/icons/kompas.svg').read_bytes())

# Preserve existing file/URI associations while hiding the old menu entry.
desktop = share / 'applications/app.shipdocs.Kompas.desktop'
legacy_desktop = share / 'applications/com.system76.CosmicStore.desktop'
legacy_desktop.write_text(desktop.read_text() + '\nNoDisplay=true\nX-AppStream-Ignore=true\n')
