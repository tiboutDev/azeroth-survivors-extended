"""Small MPQ v0 reader/writer for uncompressed, project-owned UI patches.

Storm hash/cipher algorithm; raw and single-unit raw entries only. No original
archive is rewritten. Unsupported compression/encryption fails explicitly.
"""
import struct


def crypt_table():
    seed=0x100001; table=[0]*0x500
    for i in range(256):
        for j in range(5):
            seed=(seed*125+3)%0x2aaaab; high=(seed&0xffff)<<16
            seed=(seed*125+3)%0x2aaaab; table[i+256*j]=high|(seed&0xffff)
    return table


TABLE=crypt_table()
def hash_name(name,kind):
    a=0x7fed7fed;b=0xeeeeeeee
    for c in name.replace('/','\\').upper().encode('ascii'):
        a=(TABLE[256*kind+c]^(a+b))&0xffffffff
        b=(c+a+b+(b<<5)+3)&0xffffffff
    return a


def crypt(data,key,encrypt=False):
    if len(data)%4:raise ValueError('Unaligned MPQ table')
    a=key;b=0xeeeeeeee;out=[]
    for word, in struct.iter_unpack('<I',data):
        b=(b+TABLE[0x400+(a&0xff)])&0xffffffff
        value=word^((a+b)&0xffffffff);out.append(value)
        a=(((~a<<21)+0x11111111)|(a>>11))&0xffffffff
        b=((word if encrypt else value)+b+(b<<5)+3)&0xffffffff
    return struct.pack('<'+'I'*len(out),*out)


def read_raw(path,name):
    with path.open('rb') as stream:
        header=stream.read(32)
        magic,hs,size,version,sector,hoff,boff,hcount,bcount=struct.unpack('<4sIIHHIIII',header)
        if magic!=b'MPQ\x1a' or version or hs!=32 or hcount>1_000_000 or bcount>1_000_000:
            raise ValueError('Unsupported MPQ header')
        stream.seek(hoff);hashes=crypt(stream.read(hcount*16),hash_name('(hash table)',3))
        ha,hb=hash_name(name,1),hash_name(name,2)
        for a,b,locale,platform,index in struct.iter_unpack('<IIHHI',hashes):
            if (a,b)!=(ha,hb) or locale!=0:continue
            if index>=bcount:raise ValueError('Invalid MPQ block index')
            stream.seek(boff);blocks=crypt(stream.read(bcount*16),hash_name('(block table)',3))
            off,stored,length,flags=struct.unpack_from('<IIII',blocks,index*16)
            if flags not in (0x80000000,0x81000000) or stored!=length or off+length>size:
                raise ValueError('Only complete uncompressed UI entries are supported')
            stream.seek(off);value=stream.read(length)
            if len(value)!=length:raise ValueError('Truncated MPQ file')
            return value
    raise KeyError(name)


def pack(files):
    files=dict(sorted(files.items())); count=1
    while count<max(8,len(files)*2):count*=2
    hashes=[(0xffffffff,0xffffffff,0xffff,0xffff,0xffffffff)]*count
    blocks=[];body=bytearray()
    for index,(name,value) in enumerate(files.items()):
        # The native 1.12 reader accepts ordinary raw files, but deliberately
        # rejects MPQ_FILE_SINGLE_UNIT as outside its supported envelope.
        blocks.append((32+len(body),len(value),len(value),0x80000000));body.extend(value)
        slot=hash_name(name,0)%count
        while hashes[slot][-1]!=0xffffffff:slot=(slot+1)%count
        hashes[slot]=(hash_name(name,1),hash_name(name,2),0,0,index)
    hoff=32+len(body);boff=hoff+16*count;size=boff+16*len(blocks)
    header=struct.pack('<4sIIHHIIII',b'MPQ\x1a',32,size,0,3,hoff,boff,count,len(blocks))
    return header+body+crypt(b''.join(struct.pack('<IIHHI',*h) for h in hashes),hash_name('(hash table)',3),True)+crypt(b''.join(struct.pack('<IIII',*b) for b in blocks),hash_name('(block table)',3),True)
