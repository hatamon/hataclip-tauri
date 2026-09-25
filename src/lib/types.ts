export type Item = {
  id: string;
  text: string;
  tags: string[];
  pinned: boolean;
  contexts: string[];
  pin_rank: number;
  formula?: string;
};
