import React, { useState, ChangeEvent } from "react";
import {
  TextField,
  TextFieldProps,
  StandardTextFieldProps,
  FilledTextFieldProps,
  OutlinedTextFieldProps,
} from "@material-ui/core";

type VariantTextFieldProps =
  | StandardTextFieldProps
  | FilledTextFieldProps
  | OutlinedTextFieldProps;

interface ValidatedTextFieldProps
  extends Omit<VariantTextFieldProps, "onChange" | "value"> {
  value: string;
  isValid: (value: string) => boolean;
  onValidatedChange: (value: string) => void;
  allowEmpty?: boolean;
}

export default function ValidatedTextField({
  label,
  value,
  isValid,
  onValidatedChange,
  helperText = "Invalid input",
  variant = "standard",
  allowEmpty = false,
  ...props
}: ValidatedTextFieldProps) {
  const [inputValue, setInputValue] = useState(value);
  const [errorState, setErrorState] = useState(false);

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setInputValue(newValue);

    if (newValue === "" && allowEmpty) {
      setErrorState(false);
      onValidatedChange(null);
    } else if (isValid(newValue)) {
      setErrorState(false);
      onValidatedChange(newValue);
    } else {
      setErrorState(true);
    }
  };

  return (
    <TextField
      label={label}
      value={inputValue}
      onChange={handleChange}
      error={errorState}
      helperText={errorState ? helperText : ""}
      variant={variant}
      {...props}
    />
  );
}
