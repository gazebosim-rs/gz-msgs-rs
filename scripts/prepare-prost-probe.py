#!/usr/bin/env python3
"""Prepare target-only exact crates for a bounded patch probe, never shared cache edits."""
import difflib, hashlib, pathlib, shutil, tarfile, tomllib
root=pathlib.Path(__file__).resolve().parents[1]
cache=pathlib.Path.home()/'.cargo/registry/cache'
locked=tomllib.loads((root/'experiments/prost-probe/Cargo.lock').read_text())['package']
out=root/'target/prost-probe-vendor'
out.mkdir(parents=True,exist_ok=True)
for name in ['prost-derive-0.14.4','prost-reflect-0.16.5']:
    package,version=name.rsplit('-',1)
    expected=next(p['checksum'] for p in locked if p['name']==package and p['version']==version)
    archive=next(cache.glob('*/'+name+'.crate'))
    assert hashlib.sha256(archive.read_bytes()).hexdigest()==expected
    target=out/name
    if target.exists():shutil.rmtree(target)
    with tarfile.open(archive) as source:source.extractall(out,filter='data')
    if name.startswith('prost-derive'):
        relative=pathlib.Path('src/field/scalar.rs')
        path=target/relative
        original=path.read_text()
        old='''                let default = default.typed();
                quote! {
                    if #ident != #default {'''
        new='''                let default = default.typed();
                let non_default = match self.ty {
                    Ty::Float | Ty::Double => quote!(#ident.to_bits() != (#default).to_bits()),
                    _ => quote!(#ident != #default),
                };
                quote! {
                    if #non_default {'''
        assert original.count(old)==2
        modified=original.replace(old,new)
    else:
        relative=pathlib.Path('src/dynamic/mod.rs')
        path=target/relative
        original=path.read_text()
        anchor='    /// Returns `true` if this is the default value for the given protobuf field.'
        helper='''    fn is_default_value(&self, default: Self) -> bool {
        match (self, default) {
            (Self::F32(value), Self::F32(default)) => value.to_bits() == default.to_bits(),
            (Self::F64(value), Self::F64(default)) => value.to_bits() == default.to_bits(),
            (_, default) => *self == default,
        }
    }

'''
        assert original.count(anchor)==1
        modified=original.replace(anchor,helper+anchor)
        for old,new in [('*self == Value::default_value_for_field(field_desc)','self.is_default_value(Value::default_value_for_field(field_desc))'),('*self == Value::default_value_for_extension(extension_desc)','self.is_default_value(Value::default_value_for_extension(extension_desc))'),('*self == kind.default_value()','self.is_default_value(kind.default_value())')]:
            assert modified.count(old)==1
            modified=modified.replace(old,new)
    path.write_text(modified)
    diff=''.join(difflib.unified_diff(original.splitlines(True),modified.splitlines(True),fromfile=str(relative),tofile=str(relative)))
    (root/'experiments/prost-probe'/f'{name}.patch').write_text(diff)
