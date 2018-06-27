set -eu

# List of solutions to build
WINFW_SOLUTIONS=${WINFW_SOLUTIONS:-"winfw"}
WINDNS_SOLUTIONS=${WINDNS_SOLUTIONS:-"windns"}
WINROUTE_SOLUTIONS=${WINROUTE_SOLUTIONS:-"winroute"}

# Override this variable to set your own list of build configurations. Set this
# to "Release" to build release versions.
CPP_BUILD_MODES=${CPP_BUILD_MODES:-"Debug"}
# Override this variable to set different target platforms. Add "x86" to build
# win32 versions.
CPP_BUILD_TARGETS=${CPP_BUILD_TARGETS:-"x64"}
# Override this to set a different cargo target directory
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"./target/"}


# Builds visual studio projects
function build_project
{
  local path="$1"
  local solutions="$2"

  # Sometimes the build output needs to be cleaned up
  rm -r $path/bin/* || true

  set -x
  for mode in $CPP_BUILD_MODES; do
    for target in $CPP_BUILD_TARGETS; do
      cmd.exe "/c msbuild.exe $(to_win_path $path) /p:Configuration=$mode /p:Platform=$target /t:$solutions"
    done
  done
  set +x
}

function to_win_path
{
  local unixpath=$1
  # if it's a relative path and starts with a dot (.), don't transform the
  # drive prefix (/c/ -> C:\)
  if echo $unixpath | grep '^\.' >/dev/null; then
    echo $unixpath | sed -e 's/^\///' -e 's/\//\\/g'
  # if it's an absolute path, transform the drive prefix
  else
    # remove the cygrdive prefix if it's there
    unixpath=$(echo $1 | sed -e 's/^\/cygdrive//')
    echo $unixpath | sed -e 's/^\///' -e 's/\//\\/g' -e 's/^./\0:/'
  fi
}

function get_solution_output_path
{
  local solution_root=$1
  local build_target=$2
  local build_mode=$3

  case $build_target in
    "x86")
      echo "$solution_root/bin/Win32-$build_mode"
      ;;
    "x64")
      echo "$solution_root/bin/x64-$build_mode"
      ;;
    *)
      echo Unkown build target $build_target
      exit 1
      ;;
  esac
}

# builds an appropriate cargo target path for the specified build target and
# build mode
function get_cargo_target_dir
{
  local build_target=$1
  local build_mode=$2

  local host_arch=$(rustc_host_arch)
  local host_target_arch=$(arch_from_build_target $host_arch)
  local build_target_arch=$(arch_from_build_target $build_target)
  # if the target is the same as the host, cargo omits the platform triplet
  if [ "$host_target_arch" == "$build_target_arch" ]; then
    platform_triplet=""
  # otherwise, the cargo target path is build with the platform triplet
  else
    platform_triplet="$build_target_arch-pc-windows-msvc"
  fi

  echo "$CARGO_TARGET_DIR/$platform_triplet/${build_mode,,}"
}

function copy_outputs
{
  local solution_path=$1
  local artifacts=$2

  for mode in $CPP_BUILD_MODES; do
    for target in $CPP_BUILD_TARGETS; do
      local dll_path=$(get_solution_output_path $solution_path $target $mode)
      local cargo_target=$(get_cargo_target_dir $target $mode)
      mkdir -p $cargo_target
      for artifact in $artifacts; do
        cp "$dll_path/$artifact" "$cargo_target"
      done
    done
  done

}


# Since Microsoft likes to name their architectures differently from Rust, this
# function tries to match microsoft names to Rust names.
function arch_from_build_target
{
  local build_target=$1

  case  $build_target in
    "x86")
      echo "i686"
      ;;
    "x64")
      echo "x86_64"
      ;;
    *)
      echo $build_target
      ;;
  esac
}

function rustc_host_arch
{
  rustc.exe --print cfg \
   | grep '^target_arch=' \
   | cut -d'=' -f2 \
   | tr -d '"'
}


function main
{

  local winfw_root_path=${CPP_ROOT_PATH:-"./windows/winfw"}
  local windns_root_path=${CPP_ROOT_PATH:-"./windows/windns"}
  local winroute_root_path=${CPP_ROOT_PATH:-"./windows/winroute"}

  build_project "$winfw_root_path/winfw.sln" "$WINFW_SOLUTIONS"
  build_project "$windns_root_path/windns.sln" "$WINDNS_SOLUTIONS"
  build_project "$winroute_root_path/winroute.sln" "$WINROUTE_SOLUTIONS"

  copy_outputs $winfw_root_path "winfw.dll"
  copy_outputs $windns_root_path "windns.dll"
  copy_outputs $winroute_root_path "winroute.dll"
}

main
