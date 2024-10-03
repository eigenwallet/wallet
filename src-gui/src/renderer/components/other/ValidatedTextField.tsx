import { useState, useEffect } from "react";
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
  value: string | null;
  isValid: (value: string | null) => boolean;
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

  function handleChange(newValue: string | null): void {
    newValue = newValue == null ? "" : newValue.trim();

    setInputValue(newValue);

    if (newValue === "" && allowEmpty) {
      setErrorState(false);
      onValidatedChange(null);
    } else if (newValue === "" && !allowEmpty) {
      setErrorState(true);
    } else if (isValid(newValue)) {
      setErrorState(false);
      onValidatedChange(newValue);
    } else {
      setErrorState(true);
    }
  };

  // In case the value changes from the outside, we need to update the input value
  useEffect(() => {
    handleChange(value);
  }, [value]);

  return (
    <TextField
      label={label}
      value={inputValue ?? ""}
      onChange={(e) => handleChange(e.target.value)}
      error={errorState}
      helperText={errorState ? helperText : ""}
      variant={variant}
      {...props}
    />
  );
}
