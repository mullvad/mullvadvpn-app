set -eu

# List of solutions to build
WFP_SOLUTIONS=${WFP_SOLUTIONS:-"libcommon;libwfp;wfpctl"}

# Override this variable to set your own list of build configurations for
# wfpctl
BUILD_MODES=${BUILD_MODES:-"Debug Release"}
# Override this variable to set different target platforms for wfpctl
BUILD_TARGETS=${BUILD_TARGETS:-"x86 x64"}
# Override this to set a different cargo target directory
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"./target/"}

# Builds all 4 variations of the wfpctl.dll library. Takes an argument that is
# the root of the WFP repository.
function build_wfpctl
{
	path="$1/src/wfp.sln"
	for mode in $BUILD_MODES; do
		for target in $BUILD_TARGETS; do
			echo "Running msbuild with args: $(to_win_path $path) /p:Configuration=$mode /p:Platform=$target /t:$WFP_SOLUTIONS"
			msbuild.exe "$(to_win_path $path)" \
				//p:Configuration=$mode \
				//p:Platform=$target \
				//t:$WFP_SOLUTIONS
		done
	done
}

function to_win_path
{
	echo $1 | sed -e 's/^\///' -e 's/\//\\/g' -e 's/^./\0:/'
}

function copy_outputs
{
	wfp_root_path=$1

	for mode in $BUILD_MODES; do
		for target in $BUILD_TARGETS; do
			dll_path=$(get_wfp_output_path $wfp_root_path $target $mode)
			cargo_target=$(get_cargo_target_dir $target $mode)
			mkdir -p $cargo_target
			cp "$dll_path/wfpctl.dll" $cargo_target
		done
	done

}

function get_wfp_output_path
{
	wfp_root=$1
	build_target=$2
	build_mode=$3
	case $build_target in
		"x86")
			echo "$wfp_root/bin/Win32-$build_mode"
			;;
		"x64")
			echo "$wfp_root/bin/x64-$build_mode"
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
	build_target=$1
	build_mode=$2

	host_arch=$(rustc_host_arch)
	rust_target_arch=$(arch_from_build_target $build_target)
	# if the target is the same as the host, cargo omits the platform triplet
	if [ $host_arch=$rust_target_arch ]; then
		platform_triplet=""
	# otherwise, the cargo target path is build with the platform triplet
	else
		platform_triplet="$rust_target_arch-pc-windows-msvc"
	fi

	echo "$CARGO_TARGET_DIR/$platform_triplet/${build_mode,,}"
}

# Since Microsoft likes to name their architectures differently from Rust, this
# function tries to match microsoft names to Rust names.
function arch_from_build_target
{
	build_target=$1

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
	rustc --print cfg \
	 | grep 'target_arch="x86_64"' \
	 | cut -d'=' -f2 \
	 | tr -d '"'
}


function main
{

	wfp_root_path=${WFP_ROOT_PATH:-"$(pwd)/wfp"}

	build_wfpctl $wfp_root_path
	copy_outputs $wfp_root_path
}

main
