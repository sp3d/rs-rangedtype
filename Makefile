build: librangedtype.rlib consumer

librangedtype.rlib: rangedtype.rs
	rustc $^

consumer: consumer.rs librangedtype.rlib
	rustc -L. consumer.rs

clean:
	$(RM) consumer *.rlib *.so *.dylib *.dll
