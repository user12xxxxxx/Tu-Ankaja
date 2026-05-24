export type OtpRecord = {
  otp: string;
  source_number: number;
  timestamp_micros?: number;
  created_at: string;
};

export type NumbersData = {
  numbers: number[];
  count: number;
};

export type ParamsData = {
  params: string[];
  count: number;
};

export type OtpHistoryData = {
  history: OtpRecord[];
};
