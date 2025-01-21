import { TextFieldProps, TextField } from "@material-ui/core";
import { useState, useEffect, useCallback } from "react";

interface ValidatedTextFieldProps extends Omit<TextFieldProps, "onChange" | "value"> {
  value: string | null;
  isValid: (value: string) => boolean | Promise<boolean>;
  onValidatedChange: (value: string | null) => void;
  allowEmpty?: boolean;
  noErrorWhenEmpty?: boolean;
  helperText?: string;
}

export default function ValidatedTextField({
  label,
  value = "",
  isValid,
  onValidatedChange,
  helperText = "Invalid input",
  variant = "standard",
  allowEmpty = false,
  noErrorWhenEmpty = false,
  ...props
}: ValidatedTextFieldProps) {
  const [inputValue, setInputValue] = useState(value || "");
  const [isValidating, setIsValidating] = useState(false);
  const [isError, setIsError] = useState(false);

  const handleChange = useCallback(
    async (newValue: string) => {
      const trimmedValue = newValue.trim();
      setInputValue(trimmedValue);
      
      if (trimmedValue === "" && allowEmpty) {
        setIsError(false);
        onValidatedChange(null);
        return;
      }
      
      if (trimmedValue === "" && noErrorWhenEmpty) {
        setIsError(false);
        return;
      }

      setIsValidating(true);
      try {
        const validationResult = await Promise.resolve(isValid(trimmedValue));
        setIsError(!validationResult);
        if (validationResult) {
          onValidatedChange(trimmedValue);
        }
      } finally {
        setIsValidating(false);
      }
    },
    [allowEmpty, noErrorWhenEmpty, isValid, onValidatedChange]
  );

  useEffect(() => {
    handleChange(value || "");
  }, [value]);

  return (
    <TextField
      label={label}
      value={inputValue}
      onChange={(e) => handleChange(e.target.value)}
      error={isError}
      helperText={isError ? helperText : ""}
      variant={variant}
      disabled={isValidating}
      {...props}
    />
  );
}
