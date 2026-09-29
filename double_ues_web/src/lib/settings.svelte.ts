export let settings = $state<Settings | null>(null);


type Settings = {
  username: string;
  colorHex: string;
  colorRgb: number[];
  font: string;
  pin: 'top' | 'bottom';
};
