# pyprc

A python extension module for working with Smash Ultimate parameter (".prc") files. Install using `pip install pyprc`. Packages are available for Linux, Mac, and Windows for Python 3.7+ (for 64bit versions of Python only).

## Documentation

The central export of `pyprc` is the `param` class. A param can be constructed from a file by using the default constructor and providing a filename:

```python
from pyprc import *

root = param("fighter_param.prc")
```

`pyprc` also exports a `hash` class that is used for hash-type params, described below. Hashes can be constructed from strings or from their raw integer values. Printing the string representation of a hash requires an appropriate label file. See [param-labels](https://github.com/ultimate-research/param-labels). To load labels for printing, call the `load_labels` method:

```python
h = hash("fighter_kind_pzenigame")
print(h) # prints "0x16b9c57bd9"

hash.load_labels("ParamLabels.csv")
print(h) # prints "fighter_kind_pzenigame"
```

Aside from files, params can also be constructed with static methods for each of the 12 possible types: `bool, i8, u8, i16, u16, i32, u32, float, hash, str, list, struct`. All types except `hash, list, struct` are able to be created using Python's built-in native types. Param hashes are constructed using the exported `hash` class; param lists are constructed using a list of params; and param structs are constructed with a list of hash-param tuples:

```python
p1 = param.bool(True)
p2 = param.u32(42)
p3 = param.str("woah, I'm using prc-rs from python!")

p4 = param.hash(hash("test_hash"))
p5 = param.list([
    param.u32(0),
    param.u32(45),
    param.u32(90),
])
p6 = param.struct([
    (hash("r"), param.u8(0)),
    (hash("g"), param.u8(80)),
    (hash("b"), param.u8(255)),
])
```

For all params except lists and structs, you can access and set the values with the `value` field:

```python
p = param.u8(42)
p.value += 1
print(p.value) # 43
```

Params provide instance methods that can change the identity of a param outside of just the value field. This is most useful for changing properties of param lists or structs such as length or order, but can technically be used to change any param to any other type of param. The syntax is nearly identical to the 12 param constructors, but begins with "set_", such as `set_bool` or `set_struct`, and takes the same values as the constructors:

```python
plist = param.list([param.u32(1), param.u32(2), param.u32(3)])
real_list = list(plist)
real_list.extend([param.u32(5), param.u32(8)])
plist.set_list(real_list)
for p in plist:
    print(p.value) # will print 1, 2, 3, 5, 8
```

You can access the raw param type number using the `type` field. You can compare it using the constants exported from pyprc, e.g: `PARAM_TYPE_BOOL`, `PARAM_TYPE_STRUCT`, etc.

Param lists and structs are both indexable and iterable. As such, you can write for loops over their internal params. Param lists are also convertable to Python lists, and param structs are convertable to python dicts:

```python
# access and change a param's value by indexing
param_list[0].value //= 2

# access a child param in a struct. Index must be a hash (for now)
num_jumps = fighter_data[hash("jump_count_max")]
num_jumps.value = 8

# iterate a list
for item in param_list:
    pass

# iterate a struct
for hash, item in param_struct:
    pass

# param structs contain a list of tuples instead of a dictionary because some rare param files have duplicate hashes.
# In these cases, indexing by hash will return a list containing all matching params, instead of just 1 param.
# This results in O(n) time complexity on searches. If you know that any hashes you're editing only show up once,
# consider first converting into a python dict to get O(1) search speed. See this example:

fighter_dict = dict(fighter_data)
fighter_dict[hash("attack_air_landing_frame_n")].value = 1
```

For performing a deep-copy of any data, consider using the `clone` method:

```python
# returns the index of the fighter
def get_fighter(name):
    return next(i for i, ft in enumerate(fighter_list) if ft[hash("fighter_kind")].value == hash(name))

samus = get_fighter("fighter_kind_samus")
dark_samus = get_fighter("fighter_kind_samusd")
fighter_list[dark_samus] = fighter_list[samus].clone()

# at least this stays different
fighter_list[dark_samus][hash("fighter_kind")].value = hash("fighter_kind_samusd")
```

To save a param into a file, you need a param struct as the root. Any param opened from a file will be the correct root:

```python
root = param("fighter_param.prc")
# ...
root.save("fighter_param_new.prc")
```

If for some reason you construct a param file from scratch and wish to save it, the root param is required to be a struct.

## NEW (with 1.0.0)

You can now convert strings and ints directly into the Hash class. Consider these examples:

Ex 1:

```python
# old
p_struct = param.struct([(hash("dummy"), param.u8(1))])
p_struct[hash("dummy")].value = 2

# new
p_struct = param.struct([("dummy", param.u8(1))])
p_struct["dummy"].value = 2
```

Ex 2:

```python
# old
p_hash = param.hash(hash("dummy"))
p_hash.value = hash("test")

# new
p_hash = param.hash("dummy")
p_hash.value = "test"
```
