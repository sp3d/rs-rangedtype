build: librangedtype.rlib consumer

librangedtype.rlib: rangedtype.rs
	rustc $^

consumer: consumer.rs
	rustc -L. $^

clean:
	$(RM) consumer *.rlib *.so *.dylib *.dll
