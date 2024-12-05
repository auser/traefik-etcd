import YAML from 'yaml';

export function parseYaml(yaml: string) {
  return YAML.parse(yaml);
}

export function stringifyYaml(obj: any) {
  return YAML.stringify(obj);
}