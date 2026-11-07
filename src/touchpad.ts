// Mirrors src-tauri/src/touch.rs (Settings, Calibration, Output) and extra_ipc.rs (DriverReport).
export type TouchpadStyle = 'absolute' | 'joystick' | 'swipe' | 'mouse'
export type Calibration = { xMin:number; xMax:number; yMin:number; yMax:number; forceLow:number; forceHigh:number; rotationLeft:number; rotationRight:number }
export type TouchpadSettings = { style:TouchpadStyle; clickPress:number; clickRelease:number; joystickRange:number; swipeGain:number; mouseSpeed:number; palmRejection:boolean; calibration:Calibration }
export type Contact = 'none' | 'pending' | 'active' | 'rejected'
export type PadOutput = { touch:boolean; click:boolean; x:number; y:number; force:number; contact:Contact; fingerX:number; fingerY:number; anchorX:number; anchorY:number }
export type DriverMode = 'classic' | 'fusion'
export type DriverReport = { mode:DriverMode | 'fallback' | null; adopted:[boolean,boolean]; quest3:[boolean,boolean]; conflict:string | null; skeleton:[string|null,string|null] }
export type Quest3Answer = 'ask' | 'pro' | 'ignore'

export const defaultCalibration=():Calibration=>({xMin:0,xMax:255,yMin:0,yMax:255,forceLow:0,forceHigh:1,rotationLeft:0,rotationRight:0})
export const defaultTouchpad=():TouchpadSettings=>({style:'absolute',clickPress:0.70,clickRelease:0.60,joystickRange:0.5,swipeGain:0.35,mouseSpeed:1200,palmRejection:true,calibration:defaultCalibration()})
export const idlePad=():PadOutput=>({touch:false,click:false,x:0,y:0,force:0,contact:'none',fingerX:0,fingerY:0,anchorX:0,anchorY:0})
