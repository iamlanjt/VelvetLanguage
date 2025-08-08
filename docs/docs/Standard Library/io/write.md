## Function **write**
```
write(arg as *) -> void
```


Prints a string argument to the standard output.
Similar to the `print` function, however this function prints without a newline.

This method will automatically flush stdout when done processing. For behavior that does not flush, see `write_nf`.


## Errors

Errors if writing to the stdout or flushing fails. 


<sub>internal rust::vel_write</sub>