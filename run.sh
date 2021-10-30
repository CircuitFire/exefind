#!/bin/bash 

# Directories to be scanned.
ScanDirs=(
"./"
)

# Location of exefind.
Scanner="./target/debug/exefind"

# Destination for scan results.
ScanDir="./scans"

# Destination for comparison results.
CompDir="./results"

# Make directories if they don't exist.
if [ ! -d "$ScanDir" ]; then
	mkdir -p "$ScanDir"
fi

if [ ! -d "$CompDir" ]; then
	mkdir -p "$CompDir"
fi

# Run scan on all dirs in ScanDirs
for Dir in "${ScanDirs[@]}"
do
	echo "Scanning $Dir"

	DirName=${Dir//'/'/'-'}
	DirName=${DirName/'.'/''}
	Scan="${ScanDir}/${DirName}"

	eval $Scanner -s \"$Dir\" -o \"$Scan new.csv\" > /dev/null

	if [ -f "$Scan old.csv" ]; then
		echo "Checking for changes."
		Date=$(date +%m-%d-%Y)
		eval $Scanner -c \"$Scan old.csv\" \"$Scan new.csv\" -o \"${CompDir}/${DirName} ${Date}.csv\"
	fi

	mv "$Scan new.csv" "$Scan old.csv"
done

echo "Done!"









