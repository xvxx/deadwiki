**scratch.fish**

```sh
# scratch
# usage:
#   $ scratch      - Open scratch file in vim in insert mode
#   $ scratch NOTE - Add NOTE to scratch file
function scratch
  if count $argv > /dev/null
    echo $argv >> ~/.scratch
  else
    vim " normal Go"  star ~/.scratch
  end
end
```