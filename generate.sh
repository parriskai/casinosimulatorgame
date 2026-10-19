echo "SETTING UP DIRECTORY"
mkdir -p generated-assets/
rm -rf generated-assets/*
mkdir -p generated-tiles/
rm -rf generated-tiles/*

echo "CARPET GENERATOR"
python tools/carpet_quad_to_tile.py 

echo "TILESET GENERATOR"
python3 tools/tile_generator.py
echo "DONE"