import base64
import hashlib
import json
import mimetypes
import pathlib
import urllib.error
import urllib.request
import uuid

ACCOUNT = '28a8d2ccd5db0486093cf6842dc743cd'
NAME = 'glance-landing'
ROOT = pathlib.Path(__file__).resolve().parent.parent / 'dist'
API = 'https://api.cloudflare.com/client/v4'
token = json.loads((pathlib.Path.home() / '.config/cloudflare/config/default.json').read_text())['oauth_token']

def call(method, path, token_value, data, content_type):
    request = urllib.request.Request(API + path, data=data, method=method, headers={
        'Authorization': 'Bearer ' + token_value,
        'Content-Type': content_type,
    })
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            result = json.load(response)
    except urllib.error.HTTPError as error:
        raise RuntimeError(f'{method} {path}: HTTP {error.code}: {error.read().decode()[:1000]}') from None
    if not result.get('success'):
        raise RuntimeError(f'{method} {path}: {result.get("errors")}')
    return result['result']

def multipart(parts):
    boundary = 'glance-' + uuid.uuid4().hex
    body = bytearray()
    for name, contents, mime, filename in parts:
        body.extend(f'--{boundary}\r\nContent-Disposition: form-data; name="{name}"'.encode())
        if filename:
            body.extend(f'; filename="{filename}"'.encode())
        body.extend(f'\r\nContent-Type: {mime}\r\n\r\n'.encode())
        body.extend(contents)
        body.extend(b'\r\n')
    body.extend(f'--{boundary}--\r\n'.encode())
    return bytes(body), 'multipart/form-data; boundary=' + boundary

files = {}
manifest = {}
for path in ROOT.rglob('*'):
    if not path.is_file():
        continue
    contents = path.read_bytes()
    digest = hashlib.sha256(contents).hexdigest()[:32]
    mime = mimetypes.guess_type(path.name)[0] or 'application/octet-stream'
    files[digest] = (contents, mime)
    manifest['/' + path.relative_to(ROOT).as_posix()] = {'hash': digest, 'size': len(contents)}

worker_path = f'/accounts/{ACCOUNT}/workers/scripts/{NAME}'
session = call('POST', worker_path + '/assets-upload-session', token,
               json.dumps({'manifest': manifest}).encode(), 'application/json')
upload_token = session['jwt']
for bucket in session['buckets']:
    parts = [(digest, base64.b64encode(files[digest][0]), files[digest][1], digest) for digest in bucket]
    body, content_type = multipart(parts)
    result = call('POST', f'/accounts/{ACCOUNT}/workers/assets/upload?base64=true', upload_token, body, content_type)
    if result.get('jwt'):
        upload_token = result['jwt']

metadata = {
    'main_module': 'worker.js',
    'compatibility_date': '2026-11-03',
    'assets': {'jwt': upload_token, 'config': {'html_handling': 'auto-trailing-slash', 'not_found_handling': '404-page'}},
    'bindings': [{'type': 'assets', 'name': 'ASSETS'}],
    'observability': {'enabled': True, 'traces': {'enabled': True}},
}
body, content_type = multipart([
    ('metadata', json.dumps(metadata).encode(), 'application/json', None),
    ('worker.js', b'export default { fetch(request, env) { return env.ASSETS.fetch(request); } };', 'application/javascript+module', 'worker.js'),
])
result = call('PUT', worker_path, token, body, content_type)
print('Deployed Worker:', result.get('id'), '| files:', len(manifest))
