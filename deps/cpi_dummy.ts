export type CpiDummy = {
  "version": "0.1.0",
  "name": "cpi_dummy",
  "instructions": [
    {
      "name": "msg",
      "accounts": [
        {
          "name": "dummyAcc",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    }
  ]
};

export const IDL: CpiDummy = {
  "version": "0.1.0",
  "name": "cpi_dummy",
  "instructions": [
    {
      "name": "msg",
      "accounts": [
        {
          "name": "dummyAcc",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    }
  ]
};
