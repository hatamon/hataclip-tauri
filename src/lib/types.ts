export type Item = {
  id: string;
  text: string;
  tags: string[];
  pinned: boolean;
  contexts: string[];
  paste_count: number;
  pin_rank: number;
};
