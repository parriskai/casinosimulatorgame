echo "SETTING UP DIRECTORY"
mkdir -p generated-assets/
rm -rf generated-assets/

echo "TILE GENERATOR"
python3 tools/tile_generator.py
echo "DONE"