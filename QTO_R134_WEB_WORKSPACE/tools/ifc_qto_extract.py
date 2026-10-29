"""Extract explicit IFC quantities and normalize model length/area/volume units to SI.

A verified model with explicit quantity sets is required. The script never infers
volume from tessellations or certifies an engineering quantity.
"""
import argparse
import csv
import math
from pathlib import Path

FIELDS = ['global_id', 'entity_type', 'quantity', 'unit', 'qto_set', 'property', 'source_file', 'unit_method']
KINDS = {'IfcQuantityLength': ('m', 1), 'IfcQuantityArea': ('m²', 2), 'IfcQuantityVolume': ('m³', 3), 'IfcQuantityCount': ('cái', 0)}


def extract(source: Path, destination: Path):
    try:
        import ifcopenshell
        from ifcopenshell.util.unit import calculate_unit_scale
    except ImportError as exc:
        raise RuntimeError('Cần cài IfcOpenShell và kiểm chứng trên file IFC thực') from exc
    model = ifcopenshell.open(str(source))
    # Explicit units on a quantity override project units. They are not yet
    # converted here: refuse rather than silently misstate the bill of quantities.
    project_scale = float(calculate_unit_scale(model, 'LENGTHUNIT'))
    if not math.isfinite(project_scale) or project_scale <= 0:
        raise ValueError('Đơn vị chiều dài IFC không hợp lệ')
    rows = []
    seen = set()
    for element in model.by_type('IfcElement'):
        global_id = getattr(element, 'GlobalId', None)
        if not global_id:
            continue
        for relation in getattr(element, 'IsDefinedBy', []) or []:
            if not relation.is_a('IfcRelDefinesByProperties'):
                continue
            qset = relation.RelatingPropertyDefinition
            if not qset or not qset.is_a('IfcElementQuantity'):
                continue
            for quantity in qset.Quantities:
                kind = quantity.is_a()
                if kind not in KINDS:
                    continue
                unit, power = KINDS[kind]
                if getattr(quantity, 'Unit', None):
                    raise ValueError(f'{global_id}: {quantity.Name} có đơn vị override; cần bộ chuyển đổi riêng')
                value_field = {'IfcQuantityLength': 'LengthValue', 'IfcQuantityArea': 'AreaValue',
                               'IfcQuantityVolume': 'VolumeValue', 'IfcQuantityCount': 'CountValue'}[kind]
                original = getattr(quantity, value_field, None)
                if not isinstance(original, (int, float)) or not math.isfinite(original) or original < 0:
                    raise ValueError(f'{global_id}: {quantity.Name} có giá trị không hợp lệ')
                value = original * project_scale**power
                key = (global_id, qset.Name, quantity.Name)
                if key in seen:
                    raise ValueError(f'Trùng quantity: {key}')
                seen.add(key)
                rows.append({'global_id': ':'.join(key), 'entity_type': element.is_a(), 'quantity': value,
                             'unit': unit, 'qto_set': qset.Name, 'property': quantity.Name,
                             'source_file': source.name, 'unit_method': f'project_length_scale={project_scale}'})
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open('w', newline='', encoding='utf-8-sig') as target:
        writer = csv.DictWriter(target, fieldnames=FIELDS)
        writer.writeheader()
        writer.writerows(rows)
    return len(rows)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='TBT360 IFC explicit quantities → SI CSV')
    parser.add_argument('input', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print('IFC QTO rows:', extract(args.input, args.output))
