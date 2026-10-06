```bash
result<float>
diameter<int>=2
PI<const float>=3.14

get_circumference(diameter<float>)
  circumference=PI * diameter
  echo $circumference
```

# Steps
1. Parse the script into expressions
```yml
- type: declaration
  name: result
  datatype: [float]
  lines: [1]
- type: assignment
  name: diameter
  value: 2
  datatype: [int] # inferred from the value
  lines: [2]
- type: assignment
  name: PI
  value: 3.14
  datatype: [const, float] # inferred from the value
  lines: [3]
- type: function_declaration
  name: get_circumference
  parameters:
    - name: diameter
      datatype: [float]
  lines: [5,7]
# function call not necessary due to being pure bash
```

2. Replace the expressions with their corresponding bash code
```bash
# result<float>
result=
# diameter<int>=2
declare -i diameter=2
# PI<const float>=3.14
PI=$(bc -l <<< "3.14")
# get_circumference(diameter<float>)
get_circumference(){
  local diameter=$1
  local circumference=$(bc -l <<< "$PI * $diameter")
  echo $circumference
}
```

# Notes
1. all int/float declarations should be auto-converted to `{name}=$(bc -l <<< "{expr}")` (ints and consts get the declare command)