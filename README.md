# WCU 26' Casino Simulator Game
## Summary

**TBD**

## Known bugs

- Performance
  - [x] Window lags when resizing (FIXED: Relpaced GraphicsControl::dim with an Arc<(AtomicU32, AtomicU32)> which should make reads faster)
- Rendering
  - Atlas renderer
    - [x] Atlas rendering doesnt include depth so randomly the two images swap. Run it again until it works (FIXED: Added depth test)
    - [x] Atlas rendering doesnt sort by layer causing alpha mixing problems (FIXED: Sorted by layer)
  - Font renderer
    - [x] Letters wrong size (FIXED: change variable, add correct scalling)

## Authors

-  Kai Parris (Programmer)
- Charles Rothbaum (Programmer)
- Evan Wright (Composer)
- Adrian (Artist)
