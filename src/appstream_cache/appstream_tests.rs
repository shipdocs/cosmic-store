use super::*;
use std::time::Instant;

#[test]
fn bench_yaml_parsing() {
    // A small sample YAML to test structure
    let yaml_data = r#"
---
Origin: test-origin
MediaBaseUrl: https://example.com/media
---
Type: desktop-application
ID: org.example.App1
Name:
  C: App One
Summary:
  C: The first app
Icon:
  cached:
    - name: app1_icon.png
      width: 64
      height: 64
---
Type: desktop-application
ID: org.example.App2
Name:
  C: App Two
Summary:
  C: The second app
    "#;

    let idx = AppstreamCache::default();
    let start = Instant::now();
    // Note: this uses the new signature we are about to implement
    let result = idx.parse_yaml("test.yml", yaml_data.as_bytes());
    let duration = start.elapsed();

    assert!(result.is_ok());
    let (origin, infos, _) = result.unwrap();
    assert_eq!(origin, Some("test-origin".to_string()));
    assert_eq!(infos.len(), 2);

    println!("Parsed 2 YAML documents in {:?}", duration);
}

#[test]
fn bench_xml_parsing() {
    let xml_data = r#"<?xml version="1.0"?>
<components version="0.8" origin="test-origin">
  <component type="desktop-application">
    <id>org.example.App1</id>
    <name>App One</name>
    <summary>The first app</summary>
  </component>
  <component type="desktop-application">
    <id>org.example.App2</id>
    <name>App Two</name>
    <summary>The second app</summary>
  </component>
</components>
    "#;

    let idx = AppstreamCache::default();
    let start = Instant::now();
    // Note: this uses the new signature we are about to implement
    let result = idx.parse_xml("test.xml", xml_data.as_bytes());
    let duration = start.elapsed();

    assert!(result.is_ok());
    let (origin, infos, _) = result.unwrap();
    assert_eq!(origin, Some("test-origin".to_string()));
    assert_eq!(infos.len(), 2);

    println!("Parsed 2 XML components in {:?}", duration);
}
