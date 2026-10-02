export type ToolKind =
  | 'move'
  | 'marquee_rect'
  | 'marquee_ellipse'
  | 'lasso'
  | 'polygon'
  | 'wand'
  | 'crop'
  | 'eyedropper'
  | 'spot_heal'
  | 'brush'
  | 'clone_stamp'
  | 'history_brush'
  | 'eraser'
  | 'gradient'
  | 'blur'
  | 'sharpen'
  | 'smudge'
  | 'dodge'
  | 'burn'
  | 'sponge'
  | 'type'
  | 'shape'
  | 'hand'
  | 'zoom';

export type BlendMode =
  | 'Normal'
  | 'Dissolve'
  | 'Darken'
  | 'Multiply'
  | 'Color Burn'
  | 'Linear Burn'
  | 'Darker Color'
  | 'Lighten'
  | 'Screen'
  | 'Color Dodge'
  | 'Linear Dodge (Add)'
  | 'Lighter Color'
  | 'Overlay'
  | 'Soft Light'
  | 'Hard Light'
  | 'Vivid Light'
  | 'Linear Light'
  | 'Pin Light'
  | 'Hard Mix'
  | 'Difference'
  | 'Exclusion'
  | 'Subtract'
  | 'Divide'
  | 'Hue'
  | 'Saturation'
  | 'Color'
  | 'Luminosity';

export interface LayerItem {
  id: string;
  name: string;
  isVisible: boolean;
  isGroup: boolean;
  opacity: number;
  blendMode: BlendMode;
  parentId?: string;
  hasMask: boolean;
  maskEnabled: boolean;
  adjustmentKind?: string;
  hasEffects: boolean;
}

export interface DocumentState {
  id: string;
  name: string;
  width: number;
  height: number;
  scale: number;
  panX: number;
  panY: number;
  layers: LayerItem[];
  activeLayerId: string;
  undoSteps: string[];
  redoSteps: string[];
  currentHistoryIndex: number;
}

export interface BrushSettings {
  size: number;
  hardness: number;
  opacity: number;
  flow: number;
  spacing: number;
  smoothing: number;
  pressureSize: boolean;
  pressureOpacity: boolean;
}

export interface PaletteColor {
  red: number;
  green: number;
  blue: number;
  alpha: number;
}
