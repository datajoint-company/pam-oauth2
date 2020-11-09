import pam
p = pam.pam()
print(p.authenticate('raphael','anything',service='test'))
p = pam.pam()
print(p.authenticate('raphael','jasgdjkaashgdkjashgdkjasghdkjaashgdkjasashgdjkasashgdkjasashgdjkasashgdjkasasghdkjasashgdkjsashgdkjassghdkjasasghdjkasasghdkjassghdkjassghdkjasasghajskdghasjkdgh',service='test'))
